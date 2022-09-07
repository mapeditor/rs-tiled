use std::collections::HashMap;

use xml::{attribute::OwnedAttribute, reader::XmlEvent};

use crate::{
    util::{get_attrs, parse_tag, XmlEventResult},
    Error, Properties, PropertyValue, Result,
};

pub(crate) fn parse_properties(
    parser: &mut impl Iterator<Item = XmlEventResult>,
) -> Result<Properties> {
    let mut p = HashMap::new();
    parse_tag!(parser, "properties", {
        "property" => |attrs:Vec<OwnedAttribute>| {
            let (t, v_attr, k) = get_attrs!(
                for attr in attrs {
                    Some("type") => obj_type = attr,
                    Some("value") => value = attr,
                    "name" => name = attr
                }
                (obj_type, value, name)
            );
            let t = t.unwrap_or_else(|| "string".to_owned());

            let v: String = match v_attr {
                Some(val) => val,
                None => {
                    // if the "value" attribute was missing, might be a multiline string
                    match parser.next() {
                        Some(Ok(XmlEvent::Characters(s))) => Ok(s),
                        Some(Err(err)) => Err(Error::XmlDecodingError(err)),
                        None => unreachable!(), // EndDocument or error must come first
                        _ => Err(Error::MalformedAttributes(format!("property '{}' is missing a value", k))),
                    }?
                }
            };

            p.insert(k, PropertyValue::new(t, v)?);
            Ok(())
        },
    });
    Ok(p)
}
