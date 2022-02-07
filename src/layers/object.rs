use std::{collections::HashMap};

use xml::{attribute::OwnedAttribute};

use crate::{
    parse_properties,
    util::{get_attrs, parse_tag, XmlEventResult},
    Color, LayerWrapper, Object, Properties, TiledError,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectLayerData {
    pub objects: Vec<Object>,
    pub colour: Option<Color>,
}

impl ObjectLayerData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<(ObjectLayerData, Properties), TiledError> {
        let (c, ()) = get_attrs!(
            attrs,
            optionals: [
                ("color", colour, |v:String| v.parse().ok()),
            ],
            required: [],
            // this error should never happen since there are no required attrs
            TiledError::MalformedAttributes("object group parsing error".to_string())
        );
        let mut objects = Vec::new();
        let mut properties = HashMap::new();
        parse_tag!(parser, "objectgroup", {
            "object" => |attrs| {
                objects.push(Object::new(parser, attrs)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });
        Ok((ObjectLayerData { objects, colour: c }, properties))
    }
}

pub type ObjectLayer<'map> = LayerWrapper<'map, ObjectLayerData>;
