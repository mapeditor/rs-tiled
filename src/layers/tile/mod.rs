use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    parse_properties,
    util::{get_attrs, parse_tag, XmlEventResult},
    Gid, Map, MapTilesetGid, Properties, Tile, TileId, TiledError, Tileset,
};

mod finite;
mod infinite;
mod util;

pub use finite::*;
pub use infinite::*;

/// Stores the internal tile gid about a layer tile, along with how it is flipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LayerTileData {
    /// The index of the tileset this tile's in, relative to the tile's map.
    pub(crate) tileset_index: usize,
    /// The local ID of the tile in the tileset it's in.
    pub(crate) id: TileId,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

impl LayerTileData {
    const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
    const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
    const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;
    const ALL_FLIP_FLAGS: u32 = Self::FLIPPED_HORIZONTALLY_FLAG
        | Self::FLIPPED_VERTICALLY_FLAG
        | Self::FLIPPED_DIAGONALLY_FLAG;

    /// Creates a new [`LayerTileData`] from a [`GID`] plus its flipping bits.
    pub(crate) fn from_bits(bits: u32, tilesets: &[MapTilesetGid]) -> Option<Self> {
        let flags = bits & Self::ALL_FLIP_FLAGS;
        let gid = Gid(bits & !Self::ALL_FLIP_FLAGS);
        let flip_d = flags & Self::FLIPPED_DIAGONALLY_FLAG == Self::FLIPPED_DIAGONALLY_FLAG; // Swap x and y axis (anti-diagonally) [flips over y = -x line]
        let flip_h = flags & Self::FLIPPED_HORIZONTALLY_FLAG == Self::FLIPPED_HORIZONTALLY_FLAG; // Flip tile over y axis
        let flip_v = flags & Self::FLIPPED_VERTICALLY_FLAG == Self::FLIPPED_VERTICALLY_FLAG; // Flip tile over x axis

        if gid == Gid::EMPTY {
            None
        } else {
            let (tileset_index, tileset) = crate::util::get_tileset_for_gid(tilesets, gid)?;
            let id = gid.0 - tileset.first_gid.0;

            Some(Self {
                tileset_index,
                id,
                flip_h,
                flip_v,
                flip_d,
            })
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TileLayerData {
    Finite(FiniteTileLayerData),
    Infinite(InfiniteTileLayerData),
}

impl TileLayerData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        infinite: bool,
        tilesets: &[MapTilesetGid],
    ) -> Result<(Self, Properties), TiledError> {
        let ((), (width, height)) = get_attrs!(
            attrs,
            optionals: [
            ],
            required: [
                ("width", width, |v: String| v.parse().ok()),
                ("height", height, |v: String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("layer parsing error, width and height attributes required".to_string())
        );
        let mut result = Self::Finite(Default::default());
        let mut properties = HashMap::new();
        parse_tag!(parser, "layer", {
            "data" => |attrs| {
                if infinite {
                    result = Self::Infinite(InfiniteTileLayerData::new(parser, attrs, tilesets)?);
                } else {
                    result = Self::Finite(FiniteTileLayerData::new(parser, attrs, width, height, tilesets)?);
                }
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        Ok((result, properties))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerTile<'map> {
    pub tileset: &'map Tileset,
    pub tileset_index: usize,
    pub id: TileId,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

impl<'map> LayerTile<'map> {
    pub(crate) fn from_data(data: &LayerTileData, map: &'map Map) -> Self {
        Self {
            tileset: &*map.tilesets()[data.tileset_index],
            tileset_index: data.tileset_index,
            id: data.id,
            flip_h: data.flip_h,
            flip_v: data.flip_v,
            flip_d: data.flip_d,
        }
    }

    /// Get a reference to the layer tile's referenced tile, if it exists.
    pub fn get_tile(&self) -> Option<&'map Tile> {
        self.tileset.get_tile(self.id)
    }
}

pub enum TileLayer<'map> {
    Finite(FiniteTileLayer<'map>),
    Infinite(InfiniteTileLayer<'map>),
}

impl<'map> TileLayer<'map> {
    pub(crate) fn new(map: &'map Map, data: &'map TileLayerData) -> Self {
        match data {
            TileLayerData::Finite(data) => Self::Finite(FiniteTileLayer::new(map, data)),
            TileLayerData::Infinite(data) => Self::Infinite(InfiniteTileLayer::new(map, data)),
        }
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<LayerTile> {
        match self {
            TileLayer::Finite(finite) => finite.get_tile(x, y),
            TileLayer::Infinite(infinite) => infinite.get_tile(x, y),
        }
    }
}
