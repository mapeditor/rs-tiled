use xml::attribute::OwnedAttribute;

use crate::{
    util::{get_attrs, map_wrapper, XmlEventResult},
    LayerTile, LayerTileData, MapTilesetGid, Result,
};

use super::util::parse_data_line;

#[derive(PartialEq, Clone, Default)]
pub(crate) struct FiniteTileLayerData {
    width: u32,
    height: u32,
    /// The tiles are arranged in rows.
    tiles: Vec<Option<LayerTileData>>,
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
    ) -> Result<Self> {
        let (e, c) = get_attrs!(
            attrs,
            optionals: [
                ("encoding", encoding, |v| Some(v)),
                ("compression", compression, |v| Some(v)),
            ]
        );

        let tiles = parse_data_line(e, c, parser, tilesets)?;

        Ok(Self {
            width,
            height,
            tiles,
        })
    }

    pub(crate) fn get_tile(&self, x: i32, y: i32) -> Option<&LayerTileData> {
        if x < self.width as i32 && y < self.height as i32 && x >= 0 && y >= 0 {
            self.tiles[x as usize + y as usize * self.width as usize].as_ref()
        } else {
            None
        }
    }
}

map_wrapper!(
    #[doc = "A [`TileLayer`](super::TileLayer) with a defined bound (width and height)."]
    FiniteTileLayer => FiniteTileLayerData
);

impl<'map> FiniteTileLayer<'map> {
    /// Obtains the tile present at the position given.
    ///
    /// If the position given is invalid or the position is empty, this function will return [`None`].
    pub fn get_tile(&self, x: i32, y: i32) -> Option<LayerTile> {
        self.data
            .get_tile(x, y)
            .and_then(|data| Some(LayerTile::new(self.map(), data)))
    }

    /// Get the tile layer's width in tiles.
    #[inline]
    pub fn width(&self) -> u32 {
        self.data.width
    }

    /// Get the tile layer's height in tiles.
    #[inline]
    pub fn height(&self) -> u32 {
        self.data.height
    }
}
