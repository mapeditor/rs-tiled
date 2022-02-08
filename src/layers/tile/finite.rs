use xml::attribute::OwnedAttribute;

use crate::{
    util::{get_attrs, XmlEventResult},
    LayerTileData, MapTilesetGid, TiledError,
};

use super::util::parse_data_line;

#[derive(PartialEq, Clone, Default)]
pub struct FiniteTileLayerData {
    width: u32,
    height: u32,
    /// The tiles are arranged in rows.
    pub(crate) tiles: Vec<Option<LayerTileData>>,
}

impl std::fmt::Debug for FiniteTileLayerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FiniteTileLayerData")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl FiniteTileLayerData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        width: u32,
        height: u32,
        tilesets: &[MapTilesetGid],
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

        let tiles = parse_data_line(e, c, parser, tilesets)?;

        Ok(Self {
            width,
            height,
            tiles,
        })
    }

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> Option<&LayerTileData> {
        if x < self.width as usize && y < self.height as usize {
            self.tiles[x + y * self.width as usize].as_ref()
        } else {
            None
        }
    }

    /// Get the tile layer's width in tiles.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the tile layer's height in tiles.
    pub fn height(&self) -> u32 {
        self.height
    }
}
