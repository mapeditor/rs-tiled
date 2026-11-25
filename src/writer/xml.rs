//! Keep the parts of an XML map the caller never edits, including mixed content,
//! processing instructions, namespace prefixes and whitespace.
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::{self, BufRead, Write};

pub(super) fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn validate_text(value: &str) -> io::Result<()> {
    if value.chars().any(|c| {
        c < ' ' && !matches!(c, '\n' | '\r' | '\t') || matches!(c, '\u{fffe}' | '\u{ffff}')
    }) {
        return Err(invalid("TMX text contains an invalid XML character"));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub(super) enum Node {
    Element(TmxElement),
    Content(Event<'static>),
}

impl Node {
    pub(super) fn element(&self) -> Option<&TmxElement> {
        match self {
            Self::Element(e) => Some(e),
            _ => None,
        }
    }
    pub(super) fn element_mut(&mut self) -> Option<&mut TmxElement> {
        match self {
            Self::Element(e) => Some(e),
            _ => None,
        }
    }
    pub(super) fn text(text: &str) -> Self {
        Self::Content(Event::Text(BytesText::new(text).into_owned()))
    }
    fn whitespace(&self) -> bool {
        matches!(self, Self::Content(Event::Text(t)) if t.iter().all(u8::is_ascii_whitespace))
    }
    pub(super) fn write(&self, out: &mut Writer<impl Write>) -> io::Result<()> {
        match self {
            Self::Element(e) => e.write(out),
            Self::Content(event) => {
                if let Event::Text(text) = event {
                    validate_text(std::str::from_utf8(text.as_ref()).map_err(invalid)?)?;
                }
                out.write_event(event.borrow()).map_err(io::Error::other)
            }
        }
    }
}

/// A map, layer, or object in an editable TMX document.
#[derive(Clone, Debug)]
pub struct TmxElement {
    pub(super) name: String,
    attributes: Vec<(String, String)>,
    pub(super) children: Vec<Node>,
}

impl TmxElement {
    pub(super) fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }
    /// Read a decoded XML attribute.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
    /// Set a decoded XML attribute. Escaping is applied when writing.
    pub fn set_attribute(&mut self, name: &str, value: impl ToString) {
        let value = value.to_string();
        if let Some((_, old)) = self.attributes.iter_mut().find(|(k, _)| k == name) {
            *old = value;
        } else {
            self.attributes.push((name.to_owned(), value));
        }
    }
    /// Remove an XML attribute if present.
    pub fn remove_attribute(&mut self, name: &str) {
        self.attributes.retain(|(k, _)| k != name);
    }
    pub(super) fn elements(&self) -> impl Iterator<Item = &Self> {
        self.children.iter().filter_map(Node::element)
    }
    pub(super) fn elements_mut(&mut self) -> impl Iterator<Item = &mut Self> {
        self.children.iter_mut().filter_map(Node::element_mut)
    }
    pub(super) fn child_mut(&mut self, name: &str) -> Option<&mut Self> {
        self.elements_mut().find(|e| e.name == name)
    }
    pub(super) fn append(&mut self, child: Self) {
        let i = self
            .children
            .iter()
            .rposition(|n| !n.whitespace())
            .map_or(0, |i| i + 1);
        let indent = self.children.iter().find(|n| n.whitespace()).cloned();
        if let Some(indent) = indent {
            self.children.splice(i..i, [indent, Node::Element(child)]);
        } else {
            self.children.push(Node::Element(child));
        }
    }
    pub(super) fn write(&self, out: &mut Writer<impl Write>) -> io::Result<()> {
        let mut tag = BytesStart::new(&self.name);
        for (name, value) in &self.attributes {
            if name.is_empty()
                || name.contains(|c: char| {
                    c.is_whitespace() || matches!(c, '<' | '>' | '/' | '=' | '\"' | '\'')
                })
            {
                return Err(invalid(format!("Invalid XML attribute name {name:?}")));
            }
            validate_text(value)?;
            // XML parsers normalize literal whitespace in attributes.
            let escaped = quick_xml::escape::escape(value)
                .replace('\n', "&#10;")
                .replace('\r', "&#13;")
                .replace('\t', "&#9;");
            tag.push_attribute((name.as_bytes(), escaped.as_bytes()));
        }
        if self.children.is_empty() {
            out.write_event(Event::Empty(tag)).map_err(io::Error::other)
        } else {
            out.write_event(Event::Start(tag))
                .map_err(io::Error::other)?;
            for node in &self.children {
                node.write(out)?;
            }
            out.write_event(Event::End(BytesEnd::new(&self.name)))
                .map_err(io::Error::other)
        }
    }
}

pub(super) fn read(input: impl BufRead) -> io::Result<Vec<Node>> {
    let mut parser = Reader::from_reader(input);
    let mut stack = vec![TmxElement::new("")];
    let mut buf = Vec::new();
    loop {
        let event = parser.read_event_into(&mut buf).map_err(invalid)?;
        let empty = matches!(event, Event::Empty(_));
        let node = match event {
            Event::Start(start) | Event::Empty(start) => {
                let mut elem =
                    TmxElement::new(std::str::from_utf8(start.name().as_ref()).map_err(invalid)?);
                for attr in start.attributes() {
                    let a = attr.map_err(invalid)?;
                    elem.attributes.push((
                        std::str::from_utf8(a.key.as_ref()).map_err(invalid)?.into(),
                        a.unescape_value().map_err(invalid)?.into_owned(),
                    ));
                }
                if !empty {
                    stack.push(elem);
                    buf.clear();
                    continue;
                }
                Node::Element(elem)
            }
            Event::End(_) if stack.len() > 1 => Node::Element(stack.pop().unwrap()),
            Event::End(_) => return Err(invalid("Unexpected closing tag")),
            Event::Eof if stack.len() == 1 => return Ok(stack.pop().unwrap().children),
            Event::Eof => return Err(invalid("Unexpected end of TMX document")),
            event => Node::Content(event.into_owned()),
        };
        stack.last_mut().unwrap().children.push(node);
        buf.clear();
    }
}
