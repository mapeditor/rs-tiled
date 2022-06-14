use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    error::Error,
    properties::{parse_properties, Properties},
    util::{get_attrs, parse_tag, XmlEventResult},
    Result, TileId,
};

mod wang_color;
pub use wang_color::WangColor;
mod wang_tile;
pub use wang_tile::{WangId, WangTile};

/// Undocummented WangSet types
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WangType {
    Corner,
    Edge,
    Mixed,
}

impl Default for WangType {
    fn default() -> Self {
        WangType::Mixed
    }
}

/// Raw data belonging to a WangSet.
#[derive(Debug, PartialEq, Clone)]
pub struct WangSet {
    /// The name of the Wang set
    pub name: String,
    /// Type of wangset
    pub wang_type: WangType,
    /// The tile ID of the tile representing this Wang set.
    pub tile: Option<TileId>,
    /// A color that can be used to define the corner and/or edge of a Wang tile.
    pub wang_color: Vec<WangColor>,
    /// A color that can be used to define the corner and/or edge of a Wang tile.
    pub wang_tiles: HashMap<TileId, WangTile>,
    /// The custom properties of this tile.
    pub properties: Properties,
}

impl WangSet {
    /// Reads data from XML parser to create a WangSet.
    pub fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<WangSet> {
        // Get common data
        let (name, wang_type, tile) = get_attrs!(
            for v in attrs {
                "name" => name ?= v.parse::<String>(),
                "type" => wang_type ?= v.parse::<String>(),
                "tile" => tile ?= v.parse::<i64>(),
            }
            (name, wang_type, tile)
        );

        let wang_type = match wang_type.as_str() {
            "corner" => WangType::Corner,
            "edge" => WangType::Edge,
            _ => WangType::default(),
        };
        let tile = if tile >= 0 { Some(tile as u32) } else { None };

        // Gather variable data
        let mut wang_color = Vec::new();
        let mut wang_tiles = HashMap::new();
        let mut properties = HashMap::new();
        parse_tag!(parser, "wangset", {
            "wangcolor" => |attrs| {
                let color = WangColor::new(parser, attrs)?;
                wang_color.push(color);
                Ok(())
            },
            "wangtile" => |attrs| {
                let (id, t) = WangTile::new(parser, attrs)?;
                wang_tiles.insert(id, t);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        Ok(WangSet {
            name,
            wang_type,
            tile,
            wang_color,
            wang_tiles,
            properties,
        })
    }
}
