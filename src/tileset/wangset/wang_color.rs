use std::collections::HashMap;

use crate::{
    properties::{parse_properties, Color, Properties},
    util::{get_attrs, parse_tag},
    Result, TileId,
};

/// Stores the data of the Wang color.
#[derive(Debug, PartialEq, Clone)]
pub struct WangColor {
    /// The name of this color.
    pub name: String,
    /// The custom type of this color, arbitrarily set by the user.
    pub user_type: String,
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
    pub(crate) fn new<R: std::io::BufRead>(
        elem: crate::util::XmlElement<'_, R>,
    ) -> Result<WangColor> {
        // Get common data
        let ((user_type,), (name, color, tile, probability)) = get_attrs!(
            for v in (elem.attrs) {
                Some("class") => user_type ?= v.parse::<String>(),
                "name" => name ?= v.parse::<String>(),
                "color" => color ?= v.parse(),
                "tile" => tile ?= v.parse::<i64>(),
                "probability" => probability ?= v.parse::<f32>(),
            }
            ((user_type,), (name, color, tile, probability))
        );

        let tile = if tile >= 0 { Some(tile as u32) } else { None };

        // Gather variable data
        let mut properties = HashMap::new();
        parse_tag!(elem, {
            "properties" => |elem| {
                properties = parse_properties(elem)?;
                Ok(())
            },
        });

        Ok(WangColor {
            name,
            user_type: user_type.unwrap_or_default(),
            color,
            tile,
            probability,
            properties,
        })
    }
}
