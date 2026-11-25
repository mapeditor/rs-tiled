use super::xml::{Node, TmxElement, invalid};
use crate::{Properties, PropertyValue as P};
use quick_xml::{Reader, Writer, events::Event};
use std::{io, path::Path};

impl TmxElement {
    fn read_properties(&self) -> Option<Properties> {
        let mut bytes = Vec::new();
        self.write(&mut Writer::new(&mut bytes)).ok()?;
        crate::util::parse_root_element(
            &mut Reader::from_reader(bytes.as_slice()),
            b"properties",
            crate::properties::parse_properties,
        )
        .ok()
    }

    /// Replace this element's custom properties. Unchanged values retain their original
    /// representation, including enum type metadata. File values are relative to the
    /// directory passed to [`super::TmxDocument::open`].
    pub fn set_properties(&mut self, values: &Properties) {
        let index = self
            .children
            .iter()
            .position(|n| n.element().is_some_and(|e| e.name == "properties"));
        let index = match index {
            Some(i) => i,
            None if values.is_empty() => return,
            None => {
                self.children
                    .insert(0, Node::Element(Self::new("properties")));
                0
            }
        };
        let props = self.children[index].element_mut().unwrap();
        let original = props.read_properties().unwrap_or_default();
        if original == *values {
            return;
        }
        props.children.retain(|n| {
            !n.element().is_some_and(|e| {
                e.name == "property"
                    && e.attribute("name")
                        .is_some_and(|name| !values.contains_key(name))
            })
        });
        let mut names: Vec<_> = values.keys().collect();
        names.sort_unstable();
        for name in names {
            let value = &values[name];
            if original.get(name) == Some(value) {
                continue;
            }
            if let Some(property) = props
                .elements_mut()
                .find(|e| e.name == "property" && e.attribute("name") == Some(name.as_str()))
            {
                property.set_value(value);
            } else {
                let mut property = Self::new("property");
                property.set_attribute("name", name);
                property.set_value(value);
                props.append(property);
            }
        }
        if values.is_empty() && props.children.iter().all(|n|matches!(n,Node::Content(Event::Text(t)) if t.iter().all(u8::is_ascii_whitespace))) {
            self.children.remove(index);
        }
    }

    fn set_value(&mut self, value: &P) {
        let (ty, scalar) = match value {
            P::BoolValue(v) => ("bool", Some(v.to_string())),
            P::FloatValue(v) => ("float", Some(v.to_string())),
            P::IntValue(v) => ("int", Some(v.to_string())),
            P::ColorValue(c) => (
                "color",
                Some(format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    c.alpha, c.red, c.green, c.blue
                )),
            ),
            P::StringValue(v) => ("string", Some(v.clone())),
            P::FileValue(v) => ("file", Some(v.clone())),
            P::ObjectValue(v) => ("object", Some(v.to_string())),
            P::ListValue(_) => ("list", None),
            P::ClassValue { .. } => ("class", None),
        };
        if self.attribute("type").unwrap_or("string") != ty {
            self.remove_attribute("propertytype");
            self.children.clear();
        }
        if ty == "string" {
            self.remove_attribute("type");
        } else {
            self.set_attribute("type", ty);
        }
        self.remove_attribute("value");
        if let Some(scalar) = scalar {
            self.children.clear();
            if ty == "string" && scalar.contains('\n') {
                self.children.push(Node::text(&scalar));
            } else {
                self.set_attribute("value", scalar);
            }
        } else if let P::ClassValue {
            property_type,
            properties,
        } = value
        {
            self.set_attribute("propertytype", property_type);
            self.set_properties(properties);
        } else if let P::ListValue(items) = value {
            // Keep item-specific metadata (such as enum names) when updating a list.
            let mut previous =
                std::mem::take(&mut self.children)
                    .into_iter()
                    .filter_map(|n| match n {
                        Node::Element(e) if e.name == "item" => Some(e),
                        _ => None,
                    });
            for value in items {
                let mut item = previous.next().unwrap_or_else(|| Self::new("item"));
                item.set_value(value);
                self.children.push(Node::Element(item));
            }
        }
    }

    pub(super) fn rebase(&mut self, old: &Path, new: &Path) -> io::Result<()> {
        let key = match self.name.as_str() {
            "tileset" | "image" => "source",
            "object" => "template",
            "export" => "target",
            "property" | "item" if self.attribute("type") == Some("file") => "value",
            _ => "",
        };
        if !key.is_empty() {
            if let Some(value) = self.attribute(key) {
                if let Some(path) = relative_to(value, old, new)? {
                    self.set_attribute(key, path);
                }
            } else if key == "value" {
                for child in &mut self.children {
                    let text = match child {
                        Node::Content(Event::Text(t)) => t.unescape().map_err(invalid)?,
                        Node::Content(Event::CData(data)) => {
                            std::str::from_utf8(data).map_err(invalid)?.into()
                        }
                        _ => continue,
                    };
                    if let Some(path) = relative_to(&text, old, new)? {
                        *child = Node::text(&path);
                    }
                }
            }
        }
        for child in self.elements_mut() {
            child.rebase(old, new)?;
        }
        Ok(())
    }
}

fn relative_to(value: &str, old: &Path, new: &Path) -> io::Result<Option<String>> {
    if value.is_empty() || Path::new(value).is_absolute() || value.contains("://") {
        return Ok(None);
    }
    let absolute = old.join(value);
    let path = pathdiff::diff_paths(&absolute, new).unwrap_or(absolute);
    let path = path
        .to_str()
        .ok_or_else(|| invalid("TMX paths must be UTF-8"))?;
    #[cfg(windows)]
    let path = path.replace('\\', "/");
    Ok(Some(path.to_owned()))
}
