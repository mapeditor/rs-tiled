#![feature(globs, macro_rules, struct_variant)]
extern crate flate2;
extern crate xml;
extern crate serialize;

use std::io::{BufReader, IoError, EndOfFile};
use std::from_str::FromStr;
use std::collections::HashMap;
use xml::reader::EventReader;
use xml::common::Attribute;
use xml::reader::events::*;
use serialize::base64::{FromBase64, FromBase64Error};
use flate2::reader::ZlibDecoder;

macro_rules! get_attrs {
    ($attrs:expr, optionals: [$(($oName:pat, $oVar:ident, $oT:ty, $oMethod:expr)),*], 
     required: [$(($name:pat, $var:ident, $t:ty, $method:expr)),*], $err:expr) => {
        {
            $(let mut $oVar: Option<$oT> = None;)*
            $(let mut $var: Option<$t> = None;)*
            for attr in $attrs.iter() {
                match attr.name.local_name[] {
                    $($oName => $oVar = $oMethod(attr.value.clone()),)*
                    $($name => $var = $method(attr.value.clone()),)*
                    _ => {}
                }
            }
            if !(true $(&& $var.is_some())*) {
                return Err($err);
            }
            (($($oVar),*), ($($var.unwrap()),*))
        }
    }
}

macro_rules! parse_tag {
    ($parser:expr, $close_tag:expr, $($open_tag:expr => $open_method:expr),*) => {
        loop {
            match $parser.next() {
                StartElement {name, attributes, ..} => {
                    if false {}
                    $(else if name.local_name[] == $open_tag {
                        match $open_method(attributes) {
                            Ok(()) => {},
                            Err(e) => return Err(e)
                        };
                    })*
                }
                EndElement {name, ..} => {
                    if name.local_name[] == $close_tag {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
}

#[deriving(Show)]
pub enum TiledError {
    MalformedAttributes(String),
    DecompressingError(IoError),
    DecodingError(FromBase64Error),
    Other(String)
}

pub type Properties = HashMap<String, String>;

fn parse_properties<B: Buffer>(parser: &mut EventReader<B>) -> Result<Properties, TiledError> {
    let mut p = HashMap::new();
    parse_tag!(parser, "properties",
               "property" => |attrs:Vec<Attribute>| {
                    let ((), (k, v)) = get_attrs!(
                        attrs,
                        optionals: [],
                        required: [("name", key, String, |v| Some(v)),
                                   ("value", value, String, |v| Some(v))],
                        MalformedAttributes("property must have a name and a value".to_string()));
                    p.insert(k, v);
                    Ok(())
               });
    Ok(p)
}

#[deriving(Show)]
pub struct Map {
    version: String,
    orientation: Orientation,
    width: int,
    height: int,
    tile_width: int,
    tile_height: int,
    tilesets: Vec<Tileset>,
    layers: Vec<Layer>,
    object_groups: Vec<ObjectGroup>,
    properties: Properties
}

impl Map {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<Map, TiledError>  {
        let ((), (v, o, w, h, tw, th)) = get_attrs!(
            attrs, 
            optionals: [], 
            required: [("version", version, String, |v| Some(v)),
                       ("orientation", orientation, Orientation, |v:String| from_str(v[])),
                       ("width", width, int, |v:String| from_str(v[])),
                       ("height", height, int, |v:String| from_str(v[])),
                       ("tilewidth", tile_width, int, |v:String| from_str(v[])),
                       ("tileheight", tile_height, int, |v:String| from_str(v[]))],
            MalformedAttributes("map must have a version, width and height with correct types".to_string()));

        let mut tilesets = Vec::new();
        let mut layers = Vec::new();
        let mut properties = HashMap::new();
        let mut object_groups = Vec::new();
        parse_tag!(parser, "map", 
                   "tileset" => |attrs| {
                        tilesets.push(try!(Tileset::new(parser, attrs)));
                        Ok(())
                   },
                   "layer" => |attrs| {
                        layers.push(try!(Layer::new(parser, attrs, w as uint)));
                        Ok(())
                   },
                   "properties" => |_| {
                        properties = try!(parse_properties(parser));
                        Ok(())
                   },
                   "objectgroup" => |attrs| {
                       object_groups.push(try!(ObjectGroup::new(parser, attrs)));
                       Ok(())
                   });
        Ok(Map {version: v, orientation: o,
                width: w, height: h, 
                tile_width: tw, tile_height: th,
                tilesets: tilesets, layers: layers, object_groups: object_groups,
                properties: properties})
    }

    pub fn get_tileset_by_gid(&self, gid: uint) -> Option<&Tileset> {
        let mut maximum_gid: int = -1;
        let mut maximum_ts = None;
        for tileset in self.tilesets.iter() {
            if tileset.first_gid as int > maximum_gid && tileset.first_gid < gid {
                maximum_gid = tileset.first_gid as int;
                maximum_ts = Some(tileset);
            }
        }
        maximum_ts
    }
}

#[deriving(Show)]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Staggered
}

impl FromStr for Orientation {
    fn from_str(s: &str) -> Option<Orientation> {
        match s {
            "orthogonal" => Some(Orthogonal),
            "isometric" => Some(Isometric),
            "Staggered" => Some(Staggered),
            _ => None
        }
    }
}

#[deriving(Show)]
pub struct Tileset {
    pub first_gid: uint,
    pub name: String,
    pub tile_width: uint,
    pub tile_height: uint,
    pub spacing: uint,
    pub margin: uint,
    pub images: Vec<Image>
}

impl Tileset {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<Tileset, TiledError> {
        let ((s, m), (g, n, w, h)) = get_attrs!(
           attrs,
           optionals: [("spacing", spacing, uint, |v:String| from_str(v[])),
                       ("margin", margin, uint, |v:String| from_str(v[]))],
           required: [("firstgid", first_gid, uint, |v:String| from_str(v[])),
                      ("name", name, String, |v| Some(v)),
                      ("tilewidth", width, uint, |v:String| from_str(v[])),
                      ("tileheight", height, uint, |v:String| from_str(v[]))],
           MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string()));

        let mut images = Vec::new();
        parse_tag!(parser, "tileset",
                   "image" => |attrs| {
                        images.push(try!(Image::new(parser, attrs)));
                        Ok(())
                   });
        Ok(Tileset {first_gid: g, 
                    name: n, 
                    tile_width: w, tile_height: h, 
                    spacing: s.unwrap_or(0),
                    margin: m.unwrap_or(0),
                    images: images})
   }
}

#[deriving(Show)]
pub struct Image {
    pub source: String,
    pub width: int,
    pub height: int
}

impl Image {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<Image, TiledError> {
        let ((), (s, w, h)) = get_attrs!(
            attrs,
            optionals: [],
            required: [("source", source, String, |v| Some(v)),
                       ("width", width, int, |v:String| from_str(v[])),
                       ("height", height, int, |v:String| from_str(v[]))],
            MalformedAttributes("image must have a source, width and height with correct types".to_string()));
        
        parse_tag!(parser, "image", "" => |_| Ok(()));
        Ok(Image {source: s, width: w, height: h})
    }
}

#[deriving(Show)]
pub struct Layer {
    pub name: String,
    pub opacity: f32,
    pub visible: bool,
    pub tiles: Vec<Vec<u32>>,
    pub properties: Properties
}

impl Layer {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>, width: uint) -> Result<Layer, TiledError> {
        let ((o, v), n) = get_attrs!(
            attrs,
            optionals: [("opacity", opacity, f32, |v:String| from_str(v[])),
                        ("visible", visible, bool, |v:String| from_str(v[]).map(|x:int| x == 1))],
            required: [("name", name, String, |v| Some(v))],
            MalformedAttributes("layer must have a name".to_string()));
        let mut tiles = Vec::new();
        let mut properties = HashMap::new();
        parse_tag!(parser, "layer",
                   "data" => |attrs| {
                        tiles = try!(parse_data(parser, attrs, width));
                        Ok(())
                   },
                   "properties" => |_| {
                        properties = try!(parse_properties(parser));
                        Ok(())
                   });
        Ok(Layer {name: n, opacity: o.unwrap_or(1.0), visible: v.unwrap_or(true), tiles: tiles,
                  properties: properties})
    }
}

#[deriving(Show)]
pub struct ObjectGroup {
    pub name: String,
    pub opacity: f32,
    pub visible: bool,
    pub objects: Vec<Object>
}

impl ObjectGroup {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<ObjectGroup, TiledError> {
        let ((o, v), n) = get_attrs!(
            attrs,
            optionals: [("opacity", opacity, f32, |v:String| from_str(v[])),
                        ("visible", visible, bool, |v:String| from_str(v[]).map(|x:int| x == 1))],
            required: [("name", name, String, |v| Some(v))],
            MalformedAttributes("object groups must have a name".to_string()));
        let mut objects = Vec::new();
        parse_tag!(parser, "objectgroup",
                   "object" => |attrs| {
                        objects.push(try!(Object::new(parser, attrs)));
                        Ok(())
                   });
        Ok(ObjectGroup {name: n, 
                        opacity: o.unwrap_or(1.0), visible: v.unwrap_or(true), 
                        objects: objects})
    }
}

#[deriving(Show)]
pub enum Object {
    Rect {pub x: int, pub y: int, pub width: uint, pub height: uint, pub visible: bool},
    Ellipse {pub x: int, pub y: int, pub width: uint, pub height: uint, pub visible: bool},
    Polyline {pub x: int, pub y: int, pub points: Vec<(int, int)>, pub visible: bool},
    Polygon {pub x: int, pub y: int, pub points: Vec<(int, int)>, pub visible: bool}
}

impl Object {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<Object, TiledError> {
        let ((w, h, v), (x, y)) = get_attrs!(
            attrs,
            optionals: [("width", width, uint, |v:String| from_str(v[])),
                        ("height", height, uint, |v:String| from_str(v[])),
                        ("visible", visible, bool, |v:String| from_str(v[]))],
            required: [("x", x, int, |v:String| from_str(v[])),
                       ("y", y, int, |v:String| from_str(v[]))],
            MalformedAttributes("objects must have an x and a y number".to_string()));
        let mut obj = None;
        let v = v.unwrap_or(true);
        parse_tag!(parser, "object",
                   "ellipse" => |_| {
                        if w.is_none() || h.is_none() {
                            return Err(MalformedAttributes("An ellipse must have a width and height".to_string()));
                        }
                        let (w, h) = (w.unwrap(), h.unwrap());
                        obj = Some(Ellipse {x: x, y: y, 
                                            width: w as uint, height: h as uint,
                                            visible: v});
                        Ok(())
                    },
                    "polyline" => |attrs| {
                        obj = Some(try!(Object::new_polyline(x, y, v, attrs)));
                        Ok(())
                    },
                    "polygon" => |attrs| {
                        obj = Some(try!(Object::new_polygon(x, y, v, attrs)));
                        Ok(())
                    });
        if obj.is_some() {
            Ok(obj.unwrap())
        } else if w.is_some() && h.is_some() {
            let w = w.unwrap();
            let h = h.unwrap();
            Ok(Rect {x: x, y: y, width: w as uint, height: h as uint, visible: v})
        } else {
            Err(MalformedAttributes("A rect must have a width and a height".to_string()))
        }
    }

    fn new_polyline(x: int, y: int, v: bool, attrs: Vec<Attribute>) -> Result<Object, TiledError> {
        let ((), s) = get_attrs!(
            attrs,
            optionals: [],
            required: [("points", points, String, |v| Some(v))],
            MalformedAttributes("A polyline must have points".to_string()));
       let points = try!(Object::parse_points(s));
       Ok(Polyline {x: x, y: y, points: points, visible: v})
    }

    fn new_polygon(x: int, y: int, v: bool, attrs: Vec<Attribute>) -> Result<Object, TiledError> {
        let ((), s) = get_attrs!(
            attrs,
            optionals: [],
            required: [("points", points, String, |v| Some(v))],
            MalformedAttributes("A polygon must have points".to_string()));
       let points = try!(Object::parse_points(s));
       Ok(Polygon {x: x, y: y, points: points, visible: v})
    }

    fn parse_points(s: String) -> Result<Vec<(int, int)>, TiledError> {
        let pairs = s[].split(' ');
        let mut points = Vec::new();
        for v in pairs.map(|p| p.splitn(1, ',')) {
            let v: Vec<&str> = v.clone().collect();
            if v.len() != 2 {
                return Err(MalformedAttributes("one of a polyline's points does not have an x and y coordinate".to_string()));
            }
            let (x, y) = (from_str(v[0]), from_str(v[1]));
            if x.is_none() || y.is_none() {
                return Err(MalformedAttributes("one of polyline's points does not have integer coordinates".to_string()));
            }
            points.push((x.unwrap(), y.unwrap()));
        }
        Ok(points)
    }
}

fn parse_data<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>, width: uint) -> Result<Vec<Vec<u32>>, TiledError> {
    let ((), (e, c)) = get_attrs!(
        attrs,
        optionals: [],
        required: [("encoding", encoding, String, |v| Some(v)),
                   ("compression", compression, String, |v| Some(v))],
        MalformedAttributes("data must have an encoding and a compression".to_string()));
    if !(e[] == "base64" && c[] == "zlib") {
        return Err(Other("Only base64 and zlib allowed for the moment".to_string()));
    }
    loop {
        match parser.next() {
            Characters(s) => {
                match s[].trim().from_base64() {
                    Ok(v) => {
                        let mut zd = ZlibDecoder::new(BufReader::new(v[]));
                        let mut data = Vec::new();
                        let mut row = Vec::new();
                        loop {
                            match zd.read_le_u32() {
                                Ok(v) => row.push(v),
                                Err(IoError{kind, ..}) if kind == EndOfFile => return Ok(data),
                                Err(e) => return Err(DecompressingError(e))
                            }
                            if row.len() == width {
                                data.push(row);
                                row = Vec::new();
                            }
                        }
                    }
                    Err(e) => return Err(DecodingError(e))
                }
            }
            EndElement {name, ..} => {
                if name.local_name[] == "data" {
                    return Ok(Vec::new());
                }
            }
            _ => {}
        }
    }
}

pub fn parse<B: Buffer>(parser: &mut EventReader<B>) -> Result<Map, TiledError> {
    loop {
        match parser.next() {
            StartElement {name, attributes, ..}  => {
                if name.local_name[] == "map" {
                    return Map::new(parser, attributes);
                }
            }
            _ => {}
        }
    }
}
