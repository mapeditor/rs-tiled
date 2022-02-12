use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    util::{get_attrs, parse_tag, XmlEventResult},
    LayerTile, LayerTileData, MapTilesetGid, MapWrapper, TiledError,
};

use super::util::parse_data_line;

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

        let mut chunks = HashMap::<(i32, i32), Chunk>::new();
        parse_tag!(parser, "data", {
            "chunk" => |attrs| {
                let chunk = InternalChunk::new(parser, attrs, e.clone(), c.clone(), tilesets)?;
                let first_pos = chunk.first_tile_pos;
                for x in first_pos.0..first_pos.0 + chunk.width as i32 {
                    for y in first_pos.1..first_pos.1 + chunk.height as i32{
                        let chunk_pos = tile_to_chunk_pos(x, y);
                        let relative_pos = (x - chunk_pos.0 * Chunk::WIDTH as i32, y - chunk_pos.1 * Chunk::HEIGHT as i32);
                        let chunk_index = (relative_pos.0 + relative_pos.1 * Chunk::WIDTH as i32) as usize;
                        let internal_pos = (x - first_pos.0, y - first_pos.1);
                        let internal_index = (internal_pos.0 + internal_pos.1 * chunk.width as i32) as usize;

                        chunks.entry(chunk_pos).or_insert_with(Chunk::new).tiles[chunk_index] = chunk.tiles[internal_index];
                    }
                }
                Ok(())
            }
        });

        Ok(Self { chunks })
    }

    pub(crate) fn get_tile(&self, x: i32, y: i32) -> Option<&LayerTileData> {
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

fn floor_div(a: i32, b: i32) -> i32 {
    let d = a / b;
    let r = a % b;

    if r == 0 {
        d
    } else {
        d - ((a < 0) ^ (b < 0)) as i32
    }
}

fn tile_to_chunk_pos(x: i32, y: i32) -> (i32, i32) {
    (
        floor_div(x, Chunk::WIDTH as i32),
        floor_div(y, Chunk::HEIGHT as i32),
    )
}

#[derive(Debug, PartialEq, Clone)]
pub struct Chunk {
    tiles: Vec<Option<LayerTileData>>,
}

impl Chunk {
    pub const WIDTH: u32 = 16;
    pub const HEIGHT: u32 = 16;

    pub fn new() -> Self {
        Self {
            tiles: vec![None; Self::WIDTH as usize * Self::HEIGHT as usize],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct InternalChunk {
    first_tile_pos: (i32, i32),
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
    ) -> Result<Self, TiledError> {
        let ((), (x, y, width, height)) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("x", x, |v: String| v.parse().ok()),
                ("y", y, |v: String| v.parse().ok()),
                ("width", width, |v: String| v.parse().ok()),
                ("height", height, |v: String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("layer must have a name".to_string())
        );

        let tiles = parse_data_line(encoding, compression, parser, tilesets)?;

        Ok(InternalChunk {
            first_tile_pos: (x, y),
            width,
            height,
            tiles,
        })
    }
}

pub type InfiniteTileLayer<'map> = MapWrapper<'map, InfiniteTileLayerData>;

impl<'map> InfiniteTileLayer<'map> {
    pub fn get_tile(&self, x: i32, y: i32) -> Option<LayerTile> {
        self.data()
            .get_tile(x, y)
            .and_then(|data| Some(LayerTile::from_data(data, self.map())))
    }
}
