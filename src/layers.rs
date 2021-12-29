use std::{collections::HashMap, io::Read};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    error::TiledError,
    image::Image,
    properties::{parse_properties, Properties},
    util::*,
};

/// Stores the proper tile gid, along with how it is flipped.
// Maybe PartialEq and Eq should be custom, so that it ignores tile-flipping?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerTile {
    pub gid: u32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;
const ALL_FLIP_FLAGS: u32 =
    FLIPPED_HORIZONTALLY_FLAG | FLIPPED_VERTICALLY_FLAG | FLIPPED_DIAGONALLY_FLAG;

impl LayerTile {
    pub fn new(id: u32) -> LayerTile {
        let flags = id & ALL_FLIP_FLAGS;
        let gid = id & !ALL_FLIP_FLAGS;
        let flip_d = flags & FLIPPED_DIAGONALLY_FLAG == FLIPPED_DIAGONALLY_FLAG; // Swap x and y axis (anti-diagonally) [flips over y = -x line]
        let flip_h = flags & FLIPPED_HORIZONTALLY_FLAG == FLIPPED_HORIZONTALLY_FLAG; // Flip tile over y axis
        let flip_v = flags & FLIPPED_VERTICALLY_FLAG == FLIPPED_VERTICALLY_FLAG; // Flip tile over x axis

        LayerTile {
            gid,
            flip_h,
            flip_v,
            flip_d,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Layer {
    pub name: String,
    pub opacity: f32,
    pub visible: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    /// The tiles are arranged in rows. Each tile is a number which can be used
    ///  to find which tileset it belongs to and can then be rendered.
    pub tiles: LayerData,
    pub properties: Properties,
    pub layer_index: u32,
    /// The ID of the layer, as shown in the editor.
    /// Layer ID stays the same even if layers are reordered or modified in the editor.
    pub id: u32,
}

impl Layer {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        width: u32,
        layer_index: u32,
        infinite: bool,
    ) -> Result<Layer, TiledError> {
        let ((o, v, ox, oy, n, id), ()) = get_attrs!(
            attrs,
            optionals: [
                ("opacity", opacity, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("offsetx", offset_x, |v:String| v.parse().ok()),
                ("offsety", offset_y, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
                ("id", id, |v:String| v.parse().ok()),
            ],
            required: [],
            // this error should never happen since there are no required attrs
            TiledError::MalformedAttributes("layer parsing error".to_string())
        );
        let mut tiles: LayerData = LayerData::Finite(Default::default());
        let mut properties = HashMap::new();
        parse_tag!(parser, "layer", {
            "data" => |attrs| {
                if infinite {
                    tiles = parse_infinite_data(parser, attrs, width)?;
                } else {
                    tiles = parse_data(parser, attrs, width)?;
                }
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        Ok(Layer {
            name: n.unwrap_or(String::new()),
            opacity: o.unwrap_or(1.0),
            visible: v.unwrap_or(true),
            offset_x: ox.unwrap_or(0.0),
            offset_y: oy.unwrap_or(0.0),
            tiles: tiles,
            properties: properties,
            layer_index,
            id: id.unwrap_or(0),
        })
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum LayerData {
    Finite(Vec<Vec<LayerTile>>),
    Infinite(HashMap<(i32, i32), Chunk>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImageLayer {
    pub name: String,
    pub opacity: f32,
    pub visible: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub image: Option<Image>,
    pub properties: Properties,
    pub layer_index: u32,
    /// The ID of the layer, as shown in the editor.
    /// Layer ID stays the same even if layers are reordered or modified in the editor.
    pub id: u32,
}

impl ImageLayer {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        layer_index: u32,
    ) -> Result<ImageLayer, TiledError> {
        let ((o, v, ox, oy, n, id), ()) = get_attrs!(
            attrs,
            optionals: [
                ("opacity", opacity, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("offsetx", offset_x, |v:String| v.parse().ok()),
                ("offsety", offset_y, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
                ("id", id, |v:String| v.parse().ok()),
            ],
            required: [],
            // this error should never happen since there are no required attrs
            TiledError::MalformedAttributes("image layer parsing error".to_string())
        );
        let mut properties = HashMap::new();
        let mut image: Option<Image> = None;
        parse_tag!(parser, "imagelayer", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });
        Ok(ImageLayer {
            name: n.unwrap_or(String::new()),
            opacity: o.unwrap_or(1.0),
            visible: v.unwrap_or(true),
            offset_x: ox.unwrap_or(0.0),
            offset_y: oy.unwrap_or(0.0),
            image,
            properties,
            layer_index,
            id: id.unwrap_or(0),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<LayerTile>>,
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

        let tiles = parse_data_line(encoding, compression, parser, width)?;

        Ok(Chunk {
            x,
            y,
            width,
            height,
            tiles,
        })
    }
}
