use std::{collections::HashMap, io::Read};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    parse_properties,
    util::{get_attrs, parse_tag},
    Gid, Properties, TiledError,
};

mod finite;
pub use finite::*;
mod infinite;
pub use infinite::*;

/// Stores the internal tile gid about a layer tile, along with how it is flipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LayerTileGid {
    gid: Gid,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

impl LayerTileGid {
    const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
    const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
    const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;
    const ALL_FLIP_FLAGS: u32 = Self::FLIPPED_HORIZONTALLY_FLAG
        | Self::FLIPPED_VERTICALLY_FLAG
        | Self::FLIPPED_DIAGONALLY_FLAG;

    pub(crate) fn from_bits(bits: u32) -> Self {
        let flags = bits & Self::ALL_FLIP_FLAGS;
        let gid = Gid(bits & !Self::ALL_FLIP_FLAGS);
        let flip_d = flags & Self::FLIPPED_DIAGONALLY_FLAG == Self::FLIPPED_DIAGONALLY_FLAG; // Swap x and y axis (anti-diagonally) [flips over y = -x line]
        let flip_h = flags & Self::FLIPPED_HORIZONTALLY_FLAG == Self::FLIPPED_HORIZONTALLY_FLAG; // Flip tile over y axis
        let flip_v = flags & Self::FLIPPED_VERTICALLY_FLAG == Self::FLIPPED_VERTICALLY_FLAG; // Flip tile over x axis

        Self {
            gid,
            flip_h,
            flip_v,
            flip_d,
        }
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

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> Option<&LayerTileGid> {
        match &self {
            Self::Finite(finite) => finite.get_tile(x, y),
            Self::Infinite(_) => todo!("Getting tiles from infinite layers"),
        }
    }
}
