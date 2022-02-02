use std::{collections::HashMap, io::Read};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    util::{get_attrs, parse_data_line, parse_tag},
    LayerTileData, TiledError,
};

#[derive(Debug, PartialEq, Clone)]
pub struct InfiniteTileLayerData {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl InfiniteTileLayerData {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
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
                let chunk = Chunk::new(parser, attrs, e.clone(), c.clone())?;
                chunks.insert((chunk.x, chunk.y), chunk);
                Ok(())
            }
        });

        Ok(Self { chunks })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    tiles: Vec<LayerTileData>,
}

impl Chunk {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        encoding: Option<String>,
        compression: Option<String>,
    ) -> Result<Chunk, TiledError> {
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

        let tiles = parse_data_line(encoding, compression, parser)?;

        Ok(Chunk {
            x,
            y,
            width,
            height,
            tiles,
        })
    }
}
