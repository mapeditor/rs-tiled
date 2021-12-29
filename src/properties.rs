use std::{collections::HashMap, io::Read, str::FromStr};

use xml::{EventReader, attribute::OwnedAttribute, reader::XmlEvent};

use crate::{
    error::{ParseTileError, TiledError},
    util::{get_attrs, parse_tag},
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl FromStr for Colour {
    type Err = ParseTileError;

    fn from_str(s: &str) -> Result<Colour, ParseTileError> {
        let s = if s.starts_with("#") { &s[1..] } else { s };
        if s.len() != 6 {
            return Err(ParseTileError::ColourError);
        }
        let r = u8::from_str_radix(&s[0..2], 16);
        let g = u8::from_str_radix(&s[2..4], 16);
        let b = u8::from_str_radix(&s[4..6], 16);
        if r.is_ok() && g.is_ok() && b.is_ok() {
            return Ok(Colour {
                red: r.unwrap(),
                green: g.unwrap(),
                blue: b.unwrap(),
            });
        }
        Err(ParseTileError::ColourError)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PropertyValue {
    BoolValue(bool),
    FloatValue(f32),
    IntValue(i32),
    ColorValue(u32),
    StringValue(String),
    /// Holds the path relative to the map or tileset
    FileValue(String),
    /// Holds the id of a referenced object, or 0 if unset
    ObjectValue(u32),
}

impl PropertyValue {
    fn new(property_type: String, value: String) -> Result<PropertyValue, TiledError> {
        // Check the property type against the value.
        match property_type.as_str() {
            "bool" => match value.parse() {
                Ok(val) => Ok(PropertyValue::BoolValue(val)),
                Err(err) => Err(TiledError::Other(err.to_string())),
            },
            "float" => match value.parse() {
                Ok(val) => Ok(PropertyValue::FloatValue(val)),
                Err(err) => Err(TiledError::Other(err.to_string())),
            },
            "int" => match value.parse() {
                Ok(val) => Ok(PropertyValue::IntValue(val)),
                Err(err) => Err(TiledError::Other(err.to_string())),
            },
            "color" if value.len() > 1 => match u32::from_str_radix(&value[1..], 16) {
                Ok(color) => Ok(PropertyValue::ColorValue(color)),
                Err(_) => Err(TiledError::Other(format!(
                    "Improperly formatted color property"
                ))),
            },
            "string" => Ok(PropertyValue::StringValue(value)),
            "object" => match value.parse() {
                Ok(val) => Ok(PropertyValue::ObjectValue(val)),
                Err(err) => Err(TiledError::Other(err.to_string())),
            },
            "file" => Ok(PropertyValue::FileValue(value)),
            _ => Err(TiledError::Other(format!(
                "Unknown property type \"{}\"",
                property_type
            ))),
        }
    }
}

pub type Properties = HashMap<String, PropertyValue>;

pub(crate) fn parse_properties<R: Read>(
    parser: &mut EventReader<R>,
) -> Result<Properties, TiledError> {
    let mut p = HashMap::new();
    parse_tag!(parser, "properties", {
        "property" => |attrs:Vec<OwnedAttribute>| {
            let ((t, v_attr), k) = get_attrs!(
                attrs,
                optionals: [
                    ("type", property_type, |v| Some(v)),
                    ("value", value, |v| Some(v)),
                ],
                required: [
                    ("name", key, |v| Some(v)),
                ],
                TiledError::MalformedAttributes("property must have a name and a value".to_string())
            );
            let t = t.unwrap_or("string".into());
            
            let v = match v_attr {
                Some(val) => val,
                None => {
                    // if the "value" attribute was missing, might be a multiline string
                    match parser.next().map_err(TiledError::XmlDecodingError)? {
                        XmlEvent::Characters(s) => Ok(s),
                        _ => Err(TiledError::MalformedAttributes(format!("property '{}' is missing a value", k))),
                    }?
                }
            };

            p.insert(k, PropertyValue::new(t, v)?);
            Ok(())
        },
    });
    Ok(p)
}
