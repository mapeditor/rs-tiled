use std::str::FromStr;

use xml::attribute::OwnedAttribute;

use crate::{
    error::Error,
    util::{get_attrs, XmlEventResult},
    Result, TileId,
};

/**
The Wang ID, given by a comma-separated list of indexes (starting from 1, because 0 means _unset_) referring to the Wang colors in the Wang set in the following order: top, top right, right, bottom right, bottom, bottom left, left, top left (since Tiled 1.5). Before Tiled 1.5, the Wang ID was saved as a 32-bit unsigned integer stored in the format 0xCECECECE (where each C is a corner color and each E is an edge color, in reverse order).
*/
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WangId([u32; 8]);

impl FromStr for WangId {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<WangId, Self::Err> {
        let mut ret = [0u32; 8];
        let s: Vec<&str> = s
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .collect();
        if s.len() != 8 {
            return Err(());
        }
        for i in 0..8 {
            ret[i] = s[i].parse::<u32>().unwrap_or(0);
        }

        Ok(WangId(ret))
    }
}

/// Raw data belonging to a tile.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WangTile {
    /// The tile ID.
    pub tile_id: TileId,
    /// The Wang ID,
    pub wang_id: WangId,
}

impl WangTile {
    pub fn new(
        _parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<(TileId, WangTile)> {
        // Get common data
        let (tile_id, wang_id) = get_attrs!(
            for v in attrs {
                "tileid" => tile_id ?= v.parse::<u32>(),
                "wangid" => wang_id ?= v.parse(),
            }
            (tile_id, wang_id)
        );

        Ok((tile_id, WangTile { tile_id, wang_id }))
    }
}
