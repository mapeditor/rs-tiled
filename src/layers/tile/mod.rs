use std::{collections::HashMap, io::Read};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    parse_properties,
    util::{get_attrs, parse_tag},
    Gid, LayerWrapper, MapTileset, Properties, TileId, TiledError,
};

mod finite;
mod util;
pub use finite::*;
mod infinite;
pub use infinite::*;

/// Stores the internal tile gid about a layer tile, along with how it is flipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LayerTileData {
    /// The index of the tileset this tile's in, relative to the tile's map.
    ///
    /// ## Note
    /// While the map is being parsed this is actually equal to an undefined value due to
    /// implementation-specific issues.
    tileset_index: usize,
    /// The local ID of the tile in the tileset it's in.
    ///
    /// ## Note
    /// While the map is being parsed this is actually equal to the tile's GID due to
    /// implementation-specific issues.
    id: TileId,
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

    /// Creates a new, **unfinished** [`LayerTileData`]. Unfinished layer tiles have the following
    /// properties:
    /// - `tileset_index` is set to an unspecified value.
    /// - `id` is actually the tile's GID.
    pub(crate) fn from_bits(bits: u32) -> Option<Self> {
        let flags = bits & Self::ALL_FLIP_FLAGS;
        let gid = Gid(bits & !Self::ALL_FLIP_FLAGS);
        let flip_d = flags & Self::FLIPPED_DIAGONALLY_FLAG == Self::FLIPPED_DIAGONALLY_FLAG; // Swap x and y axis (anti-diagonally) [flips over y = -x line]
        let flip_h = flags & Self::FLIPPED_HORIZONTALLY_FLAG == Self::FLIPPED_HORIZONTALLY_FLAG; // Flip tile over y axis
        let flip_v = flags & Self::FLIPPED_VERTICALLY_FLAG == Self::FLIPPED_VERTICALLY_FLAG; // Flip tile over x axis

        if gid == Gid::EMPTY {
            None
        } else {
            Some(Self {
                tileset_index: 0,
                id: gid.0,
                flip_h,
                flip_v,
                flip_d,
            })
        }
    }

    pub(crate) fn finish_details(&mut self, tilesets: &[MapTileset]) -> Result<(), ()> {
        if self.id == 0 {
            // HACK: Null tiles should be None
            self.tileset_index = 0;
            self.id = 0;
        } else {
            let (tileset_index, tileset) =
                util::get_tileset_for_gid(tilesets, Gid(self.id)).ok_or(())?;
            let id = self.id - tileset.first_gid.0;
            self.tileset_index = tileset_index;
            self.id = id;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TileLayerData {
    Finite(FiniteTileLayerData),
    Infinite(InfiniteTileLayerData),
}

impl TileLayerData {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        infinite: bool,
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
                    result = Self::Infinite(InfiniteTileLayerData::new(parser, attrs)?);
                } else {
                    result = Self::Finite(FiniteTileLayerData::new(parser, attrs, width, height)?);
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

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> Option<&LayerTileData> {
        match &self {
            Self::Finite(finite) => finite.get_tile(x, y),
            Self::Infinite(_) => todo!("Getting tiles from infinite layers"),
        }
    }
}

pub struct LayerTile<'map> {
    pub tileset: &'map MapTileset,
    pub id: TileId,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

pub type TileLayer<'map> = LayerWrapper<'map, TileLayerData>;

impl<'map> TileLayer<'map> {
    pub fn get_tile(&self, x: usize, y: usize) -> Option<LayerTile<'map>> {
        self.data.get_tile(x, y).and_then(|data| {
            Some(LayerTile {
                tileset: &self.map.tilesets()[data.tileset_index],
                id: data.id,
                flip_h: data.flip_h,
                flip_v: data.flip_v,
                flip_d: data.flip_d,
            })
        })
    }
}
