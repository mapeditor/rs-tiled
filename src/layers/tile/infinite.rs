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
    chunks: HashMap<(i32, i32), ChunkData>,
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
            for v in attrs {
                Some("encoding") => encoding = v,
                Some("compression") => compression = v,
            }
            (encoding, compression)
        );

        let mut chunks = HashMap::<(i32, i32), ChunkData>::new();
        parse_tag!(parser, "data", {
            "chunk" => |attrs| {
                let chunk = InternalChunk::new(parser, attrs, e.clone(), c.clone(), tilesets)?;
                for x in chunk.x..chunk.x + chunk.width as i32 {
                    for y in chunk.y..chunk.y + chunk.height as i32 {
                        let chunk_pos = ChunkData::tile_to_chunk_pos(x, y);
                        let relative_pos = (x - chunk_pos.0 * ChunkData::WIDTH as i32, y - chunk_pos.1 * ChunkData::HEIGHT as i32);
                        let chunk_index = (relative_pos.0 + relative_pos.1 * ChunkData::WIDTH as i32) as usize;
                        let internal_pos = (x - chunk.x, y - chunk.y);
                        let internal_index = (internal_pos.0 + internal_pos.1 * chunk.width as i32) as usize;

                        if internal_index >= chunk.tiles.len() {
                            return Err(Error::InvalidTileFound);
                        }

                        chunks.entry(chunk_pos).or_insert_with(ChunkData::new).tiles[chunk_index] = chunk.tiles[internal_index];
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
        let chunk_pos = ChunkData::tile_to_chunk_pos(x, y);
        self.chunks
            .get(&chunk_pos)
            .and_then(|chunk| {
                let relative_pos = (
                    x - chunk_pos.0 * ChunkData::WIDTH as i32,
                    y - chunk_pos.1 * ChunkData::HEIGHT as i32,
                );
                let chunk_index =
                    (relative_pos.0 + relative_pos.1 * ChunkData::WIDTH as i32) as usize;
                chunk.tiles.get(chunk_index).map(Option::as_ref)
            })
            .flatten()
    }

    /// Returns an iterator over only the data part of the chunks of this tile layer.
    ///
    /// In 99.99% of cases you'll want to use [`InfiniteTileLayer::chunks()`] instead; Using this method is only
    /// needed if you *only* require the tile data of the chunks (and no other utilities provided by
    /// the map-wrapped [`LayerTile`]), and you are in dire need for that extra bit of performance.
    ///
    /// This iterator doesn't have any particular order.
    #[inline]
    pub fn chunk_data(&self) -> impl ExactSizeIterator<Item = ((i32, i32), &ChunkData)> {
        self.chunks.iter().map(|(pos, chunk)| (*pos, chunk))
    }

    /// Obtains a chunk's data by its position. To obtain the position of the chunk that contains a
    /// tile, use [`ChunkData::tile_to_chunk_pos()`].
    ///
    /// In 99.99% of cases you'll want to use [`InfiniteTileLayer::get_chunk()`] instead; Using this method is only
    /// needed if you *only* require the tile data of the chunk (and no other utilities provided by
    /// the map-wrapped [`LayerTile`]), and you are in dire need for that extra bit of performance.
    #[inline]
    pub fn get_chunk_data(&self, x: i32, y: i32) -> Option<&ChunkData> {
        self.chunks.get(&(x, y))
    }
}

/// Part of an infinite tile layer's data.
///
/// Has only the tile data contained within and not a reference to the map it is part of.
/// In 99.99% of cases you'll actually want to use [`Chunk`].
#[derive(Debug, PartialEq, Clone)]
pub struct ChunkData {
    tiles: Box<[Option<LayerTileData>; Self::TILE_COUNT]>,
}

impl ChunkData {
    /// Infinite layer chunk width. This constant might change between versions, not counting as a
    /// breaking change.
    pub const WIDTH: u32 = 16;
    /// Infinite layer chunk height. This constant might change between versions, not counting as a
    /// breaking change.
    pub const HEIGHT: u32 = 16;
    /// Infinite layer chunk tile count. This constant might change between versions, not counting
    /// as a breaking change.
    pub const TILE_COUNT: usize = Self::WIDTH as usize * Self::HEIGHT as usize;

    pub(crate) fn new() -> Self {
        Self {
            tiles: Box::new([None; Self::TILE_COUNT]),
        }
    }

    /// Obtains the tile data present at the position given relative to the chunk's top-left-most tile.
    ///
    /// If the position given is invalid or the position is empty, this function will return [`None`].
    ///
    /// If you want to get a [`LayerTile`](`crate::LayerTile`) instead, use [`Chunk::get_tile()`].
    pub fn get_tile_data(&self, x: i32, y: i32) -> Option<&LayerTileData> {
        if x < Self::WIDTH as i32 && y < Self::HEIGHT as i32 && x >= 0 && y >= 0 {
            self.tiles[x as usize + y as usize * Self::WIDTH as usize].as_ref()
        } else {
            None
        }
    }

    /// Returns the position of the chunk that contains the given tile position.
    pub fn tile_to_chunk_pos(x: i32, y: i32) -> (i32, i32) {
        (
            floor_div(x, ChunkData::WIDTH as i32),
            floor_div(y, ChunkData::HEIGHT as i32),
        )
    }
}

map_wrapper!(
    #[doc = "Part of an [`InfiniteTileLayer`]."]
    Chunk => ChunkData
);

impl<'map> Chunk<'map> {
    /// Obtains the tile present at the position given relative to the chunk's top-left-most tile.
    ///
    /// If the position given is invalid or the position is empty, this function will return [`None`].
    pub fn get_tile(&self, x: i32, y: i32) -> Option<LayerTile<'map>> {
        self.data
            .get_tile_data(x, y)
            .map(|data| LayerTile::new(self.map(), data))
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
            for v in attrs {
                "x" => x ?= v.parse::<i32>(),
                "y" => y ?= v.parse::<i32>(),
                "width" => width ?= v.parse::<u32>(),
                "height" => height ?= v.parse::<u32>(),
            }
            (x, y, width, height)
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
    pub fn get_tile(&self, x: i32, y: i32) -> Option<LayerTile<'map>> {
        self.data
            .get_tile_data(x, y)
            .map(|data| LayerTile::new(self.map, data))
    }

    /// Returns an iterator over different parts of this map called [`Chunk`]s.
    ///
    /// These **may not** correspond with the chunks in the TMX file, as the chunk size is
    /// implementation defined (see [`ChunkData::WIDTH`], [`ChunkData::HEIGHT`]).
    ///
    /// The iterator item contains the position of the chunk in chunk coordinates along with a
    /// reference to the actual chunk at that position.
    ///
    /// This iterator doesn't have any particular order.
    ///
    /// ## Example
    /// ```
    /// # use tiled::{Loader, LayerType, TileLayer};
    /// use tiled::ChunkData;
    ///
    /// # let map = Loader::new()
    /// #     .load_tmx_map("assets/tiled_base64_zlib_infinite.tmx")
    /// #     .unwrap();
    /// # if let LayerType::Tiles(TileLayer::Infinite(infinite_layer)) =
    /// #     &map.get_layer(0).unwrap().layer_type()
    /// # {
    /// for (chunk_pos, chunk) in infinite_layer.chunks() {
    ///     for x in 0..ChunkData::WIDTH as i32 {
    ///         for y in 0..ChunkData::HEIGHT as i32 {
    ///             if let Some(tile) = chunk.get_tile(x, y) {
    ///                 let tile_pos = (
    ///                     chunk_pos.0 * ChunkData::WIDTH as i32 + x,
    ///                     chunk_pos.1 * ChunkData::HEIGHT as i32 + y,
    ///                 );
    ///                 println!("At ({}, {}): {:?}", tile_pos.0, tile_pos.1, tile);
    ///             }
    ///         }
    ///     }
    /// }
    /// # } else {
    /// #     panic!("It is wrongly recognised as a finite map");
    /// # }
    /// ```
    #[inline]
    pub fn chunks(&self) -> impl ExactSizeIterator<Item = ((i32, i32), Chunk<'map>)> + 'map {
        let map: &'map crate::Map = self.map;
        self.data
            .chunks
            .iter()
            .map(move |(pos, chunk)| (*pos, Chunk::new(map, chunk)))
    }

    /// Obtains a chunk by its position. To obtain the position of the chunk that contains a tile,
    /// use [`ChunkData::tile_to_chunk_pos()`].
    #[inline]
    pub fn get_chunk(&self, x: i32, y: i32) -> Option<Chunk<'map>> {
        let map: &'map crate::Map = self.map;
        self.data
            .get_chunk_data(x, y)
            .map(move |data| Chunk::new(map, data))
    }
}
