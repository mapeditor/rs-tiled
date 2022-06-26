use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    error::Error,
    properties::{parse_properties, Color, Properties},
    util::{get_attrs, parse_tag, XmlEventResult},
    Result, TileId,
};

/// Stores the data of the Wang color.
#[derive(Debug, PartialEq, Clone)]
pub struct WangColor {
    /// The name of this color.
    pub name: String,
    #[allow(missing_docs)]
    pub color: Color,
    /// The tile ID of the tile representing this color.
    pub tile: Option<TileId>,
    /// The relative probability that this color is chosen over others in case of multiple options. (defaults to 0)
    pub probability: f32,
    /// The custom properties of this color.
    pub properties: Properties,
}

impl WangColor {
    /// Reads data from XML parser to create a WangColor.
    pub fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<WangColor> {
        // Get common data
        let (name, color, tile, probability) = get_attrs!(
            for v in attrs {
                "name" => name ?= v.parse::<String>(),
                "color" => color ?= v.parse(),
                "tile" => tile ?= v.parse::<i64>(),
                "probability" => probability ?= v.parse::<f32>(),
            }
            (name, color, tile, probability)
        );

        let tile = if tile >= 0 { Some(tile as u32) } else { None };

        // Gather variable data
        let mut properties = HashMap::new();
        parse_tag!(parser, "wangcolor", {
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        Ok(WangColor {
            name,
            color,
            tile,
            probability,
            properties,
        })
    }
}
