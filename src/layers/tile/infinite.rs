use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    util::{floor_div, get_attrs, map_wrapper, parse_tag, XmlEventResult},
    Error, LayerTile, LayerTileData, MapTilesetGid, Result,
};

use super::util::parse_data_line;

/// The raw data of a [`InfiniteTileLayer`]. Does not include a reference to its parent [`Map`](crate::Map).
#[derive(PartialEq, Clone)]
pub struct InfiniteTileLayerData {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl std::fmt::Debug for InfiniteTileLayerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfiniteTileLayerData").finish()
    }
}

impl InfiniteTileLayerData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tilesets: &[MapTilesetGid],
    ) -> Result<Self> {
        let (e, c) = get_attrs!(
            attrs,
            optionals: [
                ("encoding", encoding, |v| Some(v)),
                ("compression", compression, |v| Some(v)),
            ]
        );

        let mut chunks = HashMap::<(i32, i32), Chunk>::new();
        parse_tag!(parser, "data", {
            "chunk" => |attrs| {
                let chunk = InternalChunk::new(parser, attrs, e.clone(), c.clone(), tilesets)?;
                for x in chunk.x..chunk.x + chunk.width as i32 {
                    for y in chunk.y..chunk.y + chunk.height as i32 {
                        let chunk_pos = tile_to_chunk_pos(x, y);
                        let relative_pos = (x - chunk_pos.0 * Chunk::WIDTH as i32, y - chunk_pos.1 * Chunk::HEIGHT as i32);
                        let chunk_index = (relative_pos.0 + relative_pos.1 * Chunk::WIDTH as i32) as usize;
                        let internal_pos = (x - chunk.x, y - chunk.y);
                        let internal_index = (internal_pos.0 + internal_pos.1 * chunk.width as i32) as usize;

                        chunks.entry(chunk_pos).or_insert_with(Chunk::new).tiles[chunk_index] = chunk.tiles[internal_index];
                    }
                }
                Ok(())
            }
        });

        Ok(Self { chunks })
    }

    /// Obtains the tile data present at the position given.
    ///
    /// If the position given is invalid or the position is empty, this function will return [`None`].
    ///
    /// If you want to get a [`Tile`](`crate::Tile`) instead, use [`InfiniteTileLayer::get_tile()`].
    pub fn get_tile_data(&self, x: i32, y: i32) -> Option<&LayerTileData> {
        let chunk_pos = tile_to_chunk_pos(x, y);
        self.chunks
            .get(&chunk_pos)
            .and_then(|chunk| {
                let relative_pos = (
                    x - chunk_pos.0 * Chunk::WIDTH as i32,
                    y - chunk_pos.1 * Chunk::HEIGHT as i32,
                );
                let chunk_index = (relative_pos.0 + relative_pos.1 * Chunk::WIDTH as i32) as usize;
                chunk.tiles.get(chunk_index).map(Option::as_ref)
            })
            .flatten()
    }
}

fn tile_to_chunk_pos(x: i32, y: i32) -> (i32, i32) {
    (
        floor_div(x, Chunk::WIDTH as i32),
        floor_div(y, Chunk::HEIGHT as i32),
    )
}

/// Part of an infinite tile layer.
#[derive(Debug, PartialEq, Clone)]
pub struct Chunk {
    tiles: Box<[Option<LayerTileData>; Self::TILE_COUNT]>,
}

impl Chunk {
    /// Internal infinite layer chunk width. Do not rely on this value as it might change between
    /// versions.
    pub const WIDTH: u32 = 16;
    /// Internal infinite layer chunk height. Do not rely on this value as it might change between
    /// versions.
    pub const HEIGHT: u32 = 16;
    /// Internal infinite layer chunk tile count. Do not rely on this value as it might change
    /// between versions.
    pub const TILE_COUNT: usize = Self::WIDTH as usize * Self::HEIGHT as usize;

    pub(crate) fn new() -> Self {
        Self {
            tiles: Box::new([None; Self::TILE_COUNT]),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct InternalChunk {
    /// The X coordinate of the top-left-most tile in the chunk.
    /// Corresponds to the `x` attribute in the TMX format.
    x: i32,
    /// The Y coordinate of the top-left-most tile in the chunk.
    /// Corresponds to the `y` attribute in the TMX format.
    y: i32,
    width: u32,
    height: u32,
    tiles: Vec<Option<LayerTileData>>,
}

impl InternalChunk {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        encoding: Option<String>,
        compression: Option<String>,
        tilesets: &[MapTilesetGid],
    ) -> Result<Self> {
        let (x, y, width, height) = get_attrs!(
            attrs,
            required: [
                ("x", x, |v: String| v.parse().ok()),
                ("y", y, |v: String| v.parse().ok()),
                ("width", width, |v: String| v.parse().ok()),
                ("height", height, |v: String| v.parse().ok()),
            ],
            Error::MalformedAttributes("chunk must have x, y, width & height attributes".to_string())
        );

        let tiles = parse_data_line(encoding, compression, parser, tilesets)?;

        Ok(InternalChunk {
            x,
            y,
            width,
            height,
            tiles,
        })
    }
}

map_wrapper!(
    #[doc = "A [`TileLayer`](super::TileLayer) with no bounds, internally stored using [`Chunk`]s."]
    InfiniteTileLayer => InfiniteTileLayerData
);

impl<'map> InfiniteTileLayer<'map> {
    /// Obtains the tile present at the position given.
    ///
    /// If the position is empty, this function will return [`None`].
    pub fn get_tile(&self, x: i32, y: i32) -> Option<LayerTile> {
        self.data
            .get_tile_data(x, y)
            .and_then(|data| Some(LayerTile::new(self.map, data)))
    }
}
