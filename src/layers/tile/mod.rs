use std::{collections::HashMap, rc::Rc};

use xml::attribute::OwnedAttribute;

use crate::{
    parse_properties,
    util::{get_attrs, parse_tag, XmlEventResult},
    Gid, MapTilesetGid, MapWrapper, Properties, TileId, TiledError, Tileset,
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
    tileset_index: usize,
    /// The local ID of the tile in the tileset it's in.
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
pub enum TileLayerData {
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

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> Option<&LayerTileData> {
        match &self {
            Self::Finite(finite) => finite.get_tile(x, y),
            Self::Infinite(_) => todo!("Getting tiles from infinite layers"),
        }
    }
}

pub struct LayerTile {
    pub tileset: Rc<Tileset>,
    pub id: TileId,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

pub type TileLayer<'map> = MapWrapper<'map, TileLayerData>;

impl<'map> TileLayer<'map> {
    pub fn get_tile(&self, x: usize, y: usize) -> Option<LayerTile> {
        self.data().get_tile(x, y).and_then(|data| {
            Some(LayerTile {
                tileset: self.map().tilesets()[data.tileset_index].clone(),
                id: data.id,
                flip_h: data.flip_h,
                flip_v: data.flip_v,
                flip_d: data.flip_d,
            })
        })
    }
}
