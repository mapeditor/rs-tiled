use std::{collections::HashMap, io::Read, path::Path};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    error::TiledError,
    image::Image,
    objects::Object,
    properties::{parse_properties, Color, Properties},
    util::*,
    Gid,
};

#[derive(Clone, PartialEq, Debug)]
pub enum LayerType {
    TileLayer(TileLayerData),
    ObjectLayer(ObjectLayer),
    ImageLayer(ImageLayer),
    // TODO: Support group layers
}

#[derive(Clone, Copy)]
pub(crate) enum LayerTag {
    TileLayer,
    ObjectLayer,
    ImageLayer,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LayerData {
    pub name: String,
    pub id: u32,
    pub visible: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub opacity: f32,
    pub properties: Properties,
    pub layer_type: LayerType,
}

impl LayerData {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        tag: LayerTag,
        infinite: bool,
        map_path: &Path,
    ) -> Result<Self, TiledError> {
        let ((opacity, visible, offset_x, offset_y, parallax_x, parallax_y, name, id), ()) = get_attrs!(
            attrs,
            optionals: [
                ("opacity", opacity, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("offsetx", offset_x, |v:String| v.parse().ok()),
                ("offsety", offset_y, |v:String| v.parse().ok()),
                ("parallaxx", parallax_x, |v:String| v.parse().ok()),
                ("parallaxy", parallax_y, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
                ("id", id, |v:String| v.parse().ok()),
            ],
            required: [
            ],

            TiledError::MalformedAttributes("layer parsing error, no id attribute found".to_string())
        );

        let (ty, properties) = match tag {
            LayerTag::TileLayer => {
                let (ty, properties) = TileLayerData::new(parser, attrs, infinite)?;
                (LayerType::TileLayer(ty), properties)
            }
            LayerTag::ObjectLayer => {
                let (ty, properties) = ObjectLayer::new(parser, attrs)?;
                (LayerType::ObjectLayer(ty), properties)
            }
            LayerTag::ImageLayer => {
                let (ty, properties) = ImageLayer::new(parser, map_path)?;
                (LayerType::ImageLayer(ty), properties)
            }
        };

        Ok(Self {
            visible: visible.unwrap_or(true),
            offset_x: offset_x.unwrap_or(0.0),
            offset_y: offset_y.unwrap_or(0.0),
            parallax_x: parallax_x.unwrap_or(1.0),
            parallax_y: parallax_y.unwrap_or(1.0),
            opacity: opacity.unwrap_or(1.0),
            name: name.unwrap_or_default(),
            id: id.unwrap_or(0),
            properties,
            layer_type: ty,
        })
    }
}

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

#[derive(Debug, PartialEq, Clone, Default)]
pub struct FiniteTileLayerData {
    width: u32,
    height: u32,
    /// The tiles are arranged in rows.
    tiles: Vec<LayerTileGid>,
}

impl FiniteTileLayerData {
    fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        width: u32,
        height: u32,
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

        let tiles = parse_data_line(e, c, parser)?;

        Ok(Self {
            width,
            height,
            tiles,
        })
    }

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> Option<&LayerTileGid> {
        if x <= self.width as usize && y <= self.height as usize {
            self.tiles.get(x + y * self.width as usize)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InfiniteTileLayerData {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl InfiniteTileLayerData {
    fn new<R: Read>(
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
pub struct ImageLayer {
    pub image: Option<Image>,
}

impl ImageLayer {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        map_path: &Path,
    ) -> Result<(ImageLayer, Properties), TiledError> {
        let mut image: Option<Image> = None;
        let mut properties = HashMap::new();

        let path_relative_to = map_path.parent().ok_or(TiledError::InvalidPath)?;

        parse_tag!(parser, "imagelayer", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, path_relative_to)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });
        Ok((ImageLayer { image }, properties))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectLayer {
    pub objects: Vec<Object>,
    pub colour: Option<Color>,
}

impl ObjectLayer {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<(ObjectLayer, Properties), TiledError> {
        let (c, ()) = get_attrs!(
            attrs,
            optionals: [
                ("color", colour, |v:String| v.parse().ok()),
            ],
            required: [],
            // this error should never happen since there are no required attrs
            TiledError::MalformedAttributes("object group parsing error".to_string())
        );
        let mut objects = Vec::new();
        let mut properties = HashMap::new();
        parse_tag!(parser, "objectgroup", {
            "object" => |attrs| {
                objects.push(Object::new(parser, attrs)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });
        Ok((
            ObjectLayer {
                objects: objects,
                colour: c,
            },
            properties,
        ))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    tiles: Vec<LayerTileGid>,
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
