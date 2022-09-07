use std::str::FromStr;

use crate::{Color, Error, PropertyValue, Result};

impl PropertyValue {
    pub(crate) fn new(property_type: String, value: String) -> Result<PropertyValue> {
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
