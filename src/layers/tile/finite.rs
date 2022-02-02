use std::io::Read;

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    util::{get_attrs, parse_data_line},
    LayerTileGid, TiledError,
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct FiniteTileLayerData {
    width: u32,
    height: u32,
    /// The tiles are arranged in rows.
    tiles: Vec<LayerTileGid>,
}

impl FiniteTileLayerData {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        width: u32,
        height: u32,
    ) -> Result<Self, TiledError> {
        let ((e, c), ()) = get_attrs!(
            attrs,
            optionals: [
                ("encoding", encoding, |v| Some(v)),
                ("compression", compression, |v| Some(v)),
            ],
            required: [],
            TiledError::MalformedAttributes("data must have an encoding and a compression".to_string())
        );

        let tiles = parse_data_line(e, c, parser)?;

        Ok(Self {
            width,
            height,
            tiles,
        })
    }

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> Option<&LayerTileGid> {
        if x <= self.width as usize && y <= self.height as usize {
            self.tiles.get(x + y * self.width as usize)
        } else {
            None
        }
    }
}
