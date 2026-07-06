use std::collections::HashMap;

use crate::{
    properties::{parse_properties, Properties},
    util::{get_attrs, parse_tag},
    Result, TileId,
};

mod wang_color;
pub use wang_color::*;
mod wang_tile;
pub use wang_tile::*;

/// Wang set's terrain brush connection type.
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(missing_docs)]
pub enum WangSetType {
    Corner,
    Edge,
    Mixed,
}

impl Default for WangSetType {
    fn default() -> Self {
        WangSetType::Mixed
    }
}

/// Raw data belonging to a WangSet.
#[derive(Debug, PartialEq, Clone)]
pub struct WangSet {
    /// The name of the Wang set.
    pub name: String,
    /// The custom type of the Wang set, arbitrarily set by the user.
    pub user_type: Option<String>,
    /// Type of Wang set.
    pub wang_set_type: WangSetType,
    /// The tile ID of the tile representing this Wang set.
    pub tile: Option<TileId>,
    /// The colors color that can be used to define the corner and/or edge of each Wang tile.
    pub wang_colors: Vec<WangColor>,
    ///  All the Wang tiles present in this Wang set, indexed by their local IDs.
    pub wang_tiles: HashMap<TileId, WangTile>,
    /// The custom properties of this Wang set.
    pub properties: Properties,
}

impl WangSet {
    /// Reads data from XML parser to create a WangSet.
    pub(crate) fn new<R: std::io::BufRead>(
        elem: crate::util::XmlElement<'_, R>,
    ) -> Result<WangSet> {
        // Get common data
        let ((user_type,), (name, wang_set_type, tile)) = get_attrs!(
            for v in (elem.attrs) {
                Some("class") => user_type ?= v.parse::<String>(),
                "name" => name ?= v.parse::<String>(),
                "type" => wang_set_type ?= v.parse::<String>(),
                "tile" => tile ?= v.parse::<i64>(),
            }
            ((user_type,), (name, wang_set_type, tile))
        );

        let wang_set_type = match wang_set_type.as_str() {
            "corner" => WangSetType::Corner,
            "edge" => WangSetType::Edge,
            _ => WangSetType::default(),
        };
        let tile = if tile >= 0 { Some(tile as u32) } else { None };

        // Gather variable data
        let mut wang_colors = Vec::new();
        let mut wang_tiles = HashMap::new();
        let mut properties = HashMap::new();
        parse_tag!(elem, {
            "wangcolor" => |elem| {
                let color = WangColor::new(elem)?;
                wang_colors.push(color);
                Ok(())
            },
            "wangtile" => |elem| {
                let (id, t) = WangTile::new(elem)?;
                wang_tiles.insert(id, t);
                Ok(())
            },
            "properties" => |elem| {
                properties = parse_properties(elem)?;
                Ok(())
            },
        });

        Ok(WangSet {
            name,
            user_type,
            wang_set_type,
            tile,
            wang_colors,
            wang_tiles,
            properties,
        })
    }
}
