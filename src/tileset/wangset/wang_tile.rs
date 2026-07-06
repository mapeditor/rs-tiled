use std::str::FromStr;

use crate::{
    Result, TileId,
    error::Error,
    util::{get_attrs, parse_tag},
};

/// The Wang ID, stored as an array of 8 u8 values.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WangId(pub [u8; 8]);

impl FromStr for WangId {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<WangId, Error> {
        let mut ret = [0u8; 8];
        let values: Vec<&str> = s
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .collect();
        if values.len() != 8 {
            return Err(Error::InvalidWangIdEncoding {
                read_string: s.to_string(),
            });
        }
        for i in 0..8 {
            ret[i] = values[i].parse::<u8>().unwrap_or(0);
        }

        Ok(WangId(ret))
    }
}

/// Stores the Wang ID.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct WangTile {
    #[allow(missing_docs)]
    pub wang_id: WangId,
}

impl WangTile {
    /// Reads data from XML parser to create a WangTile.
    pub(crate) fn new<R: std::io::BufRead>(
        elem: crate::util::XmlElement<'_, R>,
    ) -> Result<(TileId, WangTile)> {
        // Get common data
        let (tile_id, wang_id) = get_attrs!(
            for v in (elem.attrs) {
                "tileid" => tile_id ?= v.parse::<u32>(),
                "wangid" => wang_id ?= v.parse(),
            }
            (tile_id, wang_id)
        );

        parse_tag!(elem, {});
        Ok((tile_id, WangTile { wang_id }))
    }
}
