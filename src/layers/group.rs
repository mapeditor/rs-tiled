use std::io::Read;
use std::path::Path;
use std::collections::HashMap;

use xml::{attribute::OwnedAttribute, EventReader};
use crate:: {
    Layer,
    error::TiledError,
    properties::{parse_properties, Properties},
    util::*,
};

#[derive(Debug, PartialEq, Clone)]
pub struct GroupLayer<'map> {
    pub layers: Vec<Layer<'map>>,
}

impl<'map> GroupLayer<'map> {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        infinite: bool,
        path_relative_to: Option<&Path>,
    ) -> Result<(GroupLayer<'map>, Properties), TiledError> {
        let mut properties = HashMap::new();
        let mut layers = Vec::new();
        parse_tag!(parser, "group", {
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "layer" => |attrs| {
                layers.push(Layer::new(parser, attrs, LayerTag::TileLayer, infinite, source_path)?);
                Ok(())
            },
            "imagelayer" => |attrs| {
                layers.push(Layer::new(parser, attrs, LayerTag::ImageLayer, infinite, path_relative_to)?);
                Ok(())
            },
            "objectgroup" => |attrs| {
                layers.push(Layer::new(parser, attrs, LayerTag::ObjectLayer, infinite, path_relative_to)?);
                Ok(())
            },
            "group" => |attrs| {
                layers.push(Layer::new(parser, attrs, LayerTag::GroupLayer, infinite, path_relative_to)?);
                Ok(())
            },
        });
        Ok((
            GroupLayer { layers },
            properties,
        ))
    }
}