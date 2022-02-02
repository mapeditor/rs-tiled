use std::{collections::HashMap, io::Read, path::Path};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    error::TiledError,
    image::Image,
    objects::Object,
    properties::{parse_properties, Color, Properties},
    tile::Tile,
    util::*,
    Gid, Map, Tileset,
};

const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;
const ALL_FLIP_FLAGS: u32 =
    FLIPPED_HORIZONTALLY_FLAG | FLIPPED_VERTICALLY_FLAG | FLIPPED_DIAGONALLY_FLAG;

#[derive(Clone, PartialEq, Debug)]
pub enum LayerType {
    TileLayer(TileLayer),
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
pub struct Layer {
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

impl Layer {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        tag: LayerTag,
        infinite: bool,
        path_relative_to: Option<&Path>,
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
                let (ty, properties) = TileLayer::new(parser, attrs, infinite)?;
                (LayerType::TileLayer(ty), properties)
            }
            LayerTag::ObjectLayer => {
                let (ty, properties) = ObjectLayer::new(parser, attrs)?;
                (LayerType::ObjectLayer(ty), properties)
            }
            LayerTag::ImageLayer => {
                let (ty, properties) = ImageLayer::new(parser, path_relative_to)?;
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

/// Represents a tile from a tile layer.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerTileRef<'map> {
    pub tileset: &'map Tileset,
    /// The ID of the tile in its corresponding tileset.
    pub id: u32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

impl<'map> LayerTileRef<'map> {
    pub(crate) fn from_gid(layer_tile: &LayerTileGid, map: &'map Map) -> Option<Self> {
        if layer_tile.gid == Gid::EMPTY {
            None
        } else {
            map.get_tileset_for_gid(layer_tile.gid)
                .map(|tileset| Self {
                    tileset,
                    id: layer_tile.gid.0 - tileset.first_gid,
                    flip_h: layer_tile.flip_h,
                    flip_v: layer_tile.flip_v,
                    flip_d: layer_tile.flip_d,
                })
        }
    }

    pub fn tile(&self) -> Option<&Tile> {
        self.tileset.get_tile(self.id)
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
    pub(crate) fn from_bits(bits: u32) -> Self {
        let flags = bits & ALL_FLIP_FLAGS;
        let gid = Gid(bits & !ALL_FLIP_FLAGS);
        let flip_d = flags & FLIPPED_DIAGONALLY_FLAG == FLIPPED_DIAGONALLY_FLAG; // Swap x and y axis (anti-diagonally) [flips over y = -x line]
        let flip_h = flags & FLIPPED_HORIZONTALLY_FLAG == FLIPPED_HORIZONTALLY_FLAG; // Flip tile over y axis
        let flip_v = flags & FLIPPED_VERTICALLY_FLAG == FLIPPED_VERTICALLY_FLAG; // Flip tile over x axis

        Self {
            gid,
            flip_h,
            flip_v,
            flip_d,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TileLayer {
    pub width: u32,
    pub height: u32,
    /// The tiles are arranged in rows. Each tile is a number which can be used
    ///  to find which tileset it belongs to and can then be rendered.
    tiles: LayerData,
}

impl TileLayer {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        infinite: bool,
    ) -> Result<(TileLayer, Properties), TiledError> {
        let ((), (w, h)) = get_attrs!(
            attrs,
            optionals: [
            ],
            required: [
                ("width", width, |v: String| v.parse().ok()),
                ("height", height, |v: String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("layer parsing error, width and height attributes required".to_string())
        );
        let mut tiles: LayerData = LayerData::Finite(Default::default());
        let mut properties = HashMap::new();
        parse_tag!(parser, "layer", {
            "data" => |attrs| {
                if infinite {
                    tiles = parse_infinite_data(parser, attrs)?;
                } else {
                    tiles = parse_data(parser, attrs)?;
                }
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        Ok((
            TileLayer {
                width: w,
                height: h,
                tiles: tiles,
            },
            properties,
        ))
    }

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> Option<&LayerTileGid> {
        if x <= self.width as usize && y <= self.height as usize {
            match &self.tiles {
                LayerData::Finite(tiles) => tiles.get(x + y * self.width as usize),
                LayerData::Infinite(_) => todo!("Getting tiles from infinite layers"),
            }
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum LayerData {
    Finite(Vec<LayerTileGid>),
    Infinite(HashMap<(i32, i32), Chunk>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImageLayer {
    pub image: Option<Image>,
}

impl ImageLayer {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        path_relative_to: Option<&Path>,
    ) -> Result<(ImageLayer, Properties), TiledError> {
        let mut image: Option<Image> = None;
        let mut properties = HashMap::new();

        parse_tag!(parser, "imagelayer", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, path_relative_to.ok_or(TiledError::SourceRequired{object_to_parse: "Image".to_string()})?)?);
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
