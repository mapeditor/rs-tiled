use std::{collections::HashMap, str::FromStr};

use crate::{
    error::{Error, Result},
    util::{get_attrs, parse_tag, read_text_or_cdata},
};

/// Represents a RGBA color with 8-bit depth on each channel.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(missing_docs)]
pub struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Color, Self::Err> {
        let s = if let Some(stripped) = s.strip_prefix('#') {
            stripped
        } else {
            s
        };
        match s.len() {
            6 if s.is_ascii() => {
                let r = u8::from_str_radix(&s[0..2], 16);
                let g = u8::from_str_radix(&s[2..4], 16);
                let b = u8::from_str_radix(&s[4..6], 16);
                match (r, g, b) {
                    (Ok(red), Ok(green), Ok(blue)) => Ok(Color {
                        alpha: 0xFF,
                        red,
                        green,
                        blue,
                    }),
                    _ => Err(()),
                }
            }
            8 if s.is_ascii() => {
                let a = u8::from_str_radix(&s[0..2], 16);
                let r = u8::from_str_radix(&s[2..4], 16);
                let g = u8::from_str_radix(&s[4..6], 16);
                let b = u8::from_str_radix(&s[6..8], 16);
                match (a, r, g, b) {
                    (Ok(alpha), Ok(red), Ok(green), Ok(blue)) => Ok(Color {
                        alpha,
                        red,
                        green,
                        blue,
                    }),
                    _ => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}

/// Represents a custom property's value.
///
/// Also read the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tmx-properties).
#[derive(Debug, PartialEq, Clone)]
pub enum PropertyValue {
    /// A boolean value. Corresponds to the `bool` property type.
    BoolValue(bool),
    /// A floating point value. Corresponds to the `float` property type.
    FloatValue(f32),
    /// A signed integer value. Corresponds to the `int` property type.
    IntValue(i32),
    /// A color value. Corresponds to the `color` property type.
    ColorValue(Color),
    /// A string value. Corresponds to the `string` property type.
    StringValue(String),
    /// A filepath value. Corresponds to the `file` property type.
    /// Holds the path relative to the map or tileset.
    FileValue(String),
    /// An object ID value. Corresponds to the `object` property type.
    /// Holds the id of a referenced object, or 0 if unset.
    ObjectValue(u32),
    /// A list of property values. Corresponds to the `list` property type.
    ListValue(Vec<PropertyValue>),
    /// A class value. Corresponds to the `class` property type.
    /// Holds the type name and a set of properties.
    ClassValue {
        /// The type name.
        property_type: String,
        /// A set of properties.
        properties: Properties,
    },
}

impl PropertyValue {
    fn new(property_type: String, value: String) -> Result<PropertyValue> {
        // Check the property type against the value.
        match property_type.as_str() {
            "bool" => match value.parse() {
                Ok(val) => Ok(PropertyValue::BoolValue(val)),
                Err(err) => Err(Error::InvalidPropertyValue {
                    description: err.to_string(),
                }),
            },
            "float" => match value.parse() {
                Ok(val) => Ok(PropertyValue::FloatValue(val)),
                Err(err) => Err(Error::InvalidPropertyValue {
                    description: err.to_string(),
                }),
            },
            "int" => match value.parse() {
                Ok(val) => Ok(PropertyValue::IntValue(val)),
                Err(err) => Err(Error::InvalidPropertyValue {
                    description: err.to_string(),
                }),
            },
            "color" if value.len() > 1 => Color::from_str(&value)
                .map(PropertyValue::ColorValue)
                .map_err(|_| Error::InvalidPropertyValue {
                    description: "Couldn't parse color".to_string(),
                }),
            "string" => Ok(PropertyValue::StringValue(value)),
            "object" => match value.parse() {
                Ok(val) => Ok(PropertyValue::ObjectValue(val)),
                Err(err) => Err(Error::InvalidPropertyValue {
                    description: err.to_string(),
                }),
            },
            "file" => Ok(PropertyValue::FileValue(value)),
            _ => Err(Error::UnknownPropertyType {
                type_name: property_type,
            }),
        }
    }
}

/// A custom property container.
pub type Properties = HashMap<String, PropertyValue>;

pub(crate) fn parse_properties<R: std::io::BufRead>(
    elem: crate::util::XmlElement<'_, R>,
) -> Result<Properties> {
    let mut p = HashMap::new();
    parse_tag!(elem, {
        "property" => |elem: crate::util::XmlElement<'_, R>| {
            let (t, v_attr, k, p_t) = get_attrs!(
                for attr in (elem.attrs) {
                    Some("type") => obj_type = attr.to_string(),
                    Some("value") => value = attr.to_string(),
                    Some("propertytype") => propertytype = attr.to_string(),
                    "name" => name = attr.to_string()
                }
                (obj_type, value, name, propertytype)
            );
            p.insert(k, parse_property_value(elem, t, v_attr, p_t)?);
            Ok(())
        },
    });
    Ok(p)
}

/// Parses the value of a `<property>` or list `<item>` element, given its `type`, `value` and
/// `propertytype` attributes. Consumes the element's content.
fn parse_property_value<R: std::io::BufRead>(
    elem: crate::util::XmlElement<'_, R>,
    t: Option<String>,
    v_attr: Option<String>,
    p_t: Option<String>,
) -> Result<PropertyValue> {
    let t = t.unwrap_or_else(|| "string".to_owned());
    if t == "class" {
        // Class properties will have their member values stored in a nested <properties>
        // element. Only the actually set members are saved. When no members have been set
        // the properties element is left out entirely.
        let mut properties = HashMap::new();
        parse_tag!(elem, {
            "properties" => |elem| {
                properties = parse_properties(elem)?;
                Ok(())
            },
        });
        return Ok(PropertyValue::ClassValue {
            property_type: p_t.unwrap_or_default(),
            properties,
        });
    }

    if t == "list" {
        // List properties store each of their values in a nested <item> element, which is
        // structured like a <property> element without a name.
        let mut items = Vec::new();
        parse_tag!(elem, {
            "item" => |elem: crate::util::XmlElement<'_, R>| {
                let (t, v_attr, p_t) = get_attrs!(
                    for attr in (elem.attrs) {
                        Some("type") => obj_type = attr.to_string(),
                        Some("value") => value = attr.to_string(),
                        Some("propertytype") => propertytype = attr.to_string(),
                    }
                    (obj_type, value, propertytype)
                );
                items.push(parse_property_value(elem, t, v_attr, p_t)?);
                Ok(())
            },
        });
        return Ok(PropertyValue::ListValue(items));
    }

    let v: String = match v_attr {
        Some(val) => {
            parse_tag!(elem, {});
            val
        }
        None => {
            // if the "value" attribute was missing, might be a multiline string
            read_text_or_cdata(elem, |text| Ok(text.to_string()))?
        }
    };
    PropertyValue::new(t, v)
}
