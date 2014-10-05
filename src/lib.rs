#![feature(globs, macro_rules)]
extern crate flate2;
extern crate xml;
extern crate serialize;

use std::io::{BufReader, IoError, EndOfFile};
use xml::reader::EventReader;
use xml::common::Attribute;
use xml::reader::events::*;
use serialize::base64::FromBase64;
use flate2::reader::ZlibDecoder;

macro_rules! get_attrs {
    ($attrs:expr, optionals: [$(($oName:pat, $oVar:ident, $oT:ty, $oMethod:expr)),*], 
     required: [$(($name:pat, $var:ident, $t:ty, $method:expr)),*], $msg:expr) => {
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
                return Err($msg);
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
pub struct Map {
    version: String,
    width: int,
    height: int,
    tile_width: int,
    tile_height: int,
    tilesets: Vec<Tileset>,
    layers: Vec<Layer>
}

impl Map {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<Map, String>  {
        let ((), (v, w, h, tw, th)) = get_attrs!(
            attrs, 
            optionals: [], 
            required: [("version", version, String, |v| Some(v)),
                       ("width", width, int, |v:String| from_str(v[])),
                       ("height", height, int, |v:String| from_str(v[])),
                       ("tilewidth", tile_width, int, |v:String| from_str(v[])),
                       ("tileheight", tile_height, int, |v:String| from_str(v[]))],
            "map must have a version, width and height with correct types".to_string());

        let mut tilesets = Vec::new();
        let mut layers = Vec::new();
        parse_tag!(parser, "map", 
                   "tileset" => |attrs| {
                        tilesets.push(try!(Tileset::new(parser, attrs)));
                        Ok(())
                   },
                   "layer" => |attrs| {
                        layers.push(try!(Layer::new(parser, attrs, w as uint)));
                        Ok(())
                   });
        Ok(Map {version: v, 
                width: w, height: h, 
                tile_width: tw, tile_height: th,
                tilesets: tilesets, layers: layers})
    }
}

#[deriving(Show)]
pub struct Tileset {
    first_gid: int,
    name: String,
    images: Vec<Image>
}

impl Tileset {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<Tileset, String> {
        let ((), (g, n)) = get_attrs!(
           attrs,
           optionals: [],
           required: [("firstgid", first_gid, int, |v:String| from_str(v[])),
                      ("name", name, String, |v| Some(v))],
           "tileset must have a firstgid and name with correct types".to_string());

        let mut images = Vec::new();
        parse_tag!(parser, "tileset",
                   "image" => |attrs| {
                        images.push(try!(Image::new(parser, attrs)));
                        Ok(())
                   });
        Ok(Tileset {first_gid: g, name: n, images: images})
   }
}

#[deriving(Show)]
pub struct Image {
    source: String,
    width: int,
    height: int
}

impl Image {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<Image, String> {
        let ((), (s, w, h)) = get_attrs!(
            attrs,
            optionals: [],
            required: [("source", source, String, |v| Some(v)),
                       ("width", width, int, |v:String| from_str(v[])),
                       ("height", height, int, |v:String| from_str(v[]))],
            "image must have a source, width and height with correct types".to_string());
        
        parse_tag!(parser, "image", "" => |_| Ok(()));
        Ok(Image {source: s, width: w, height: h})
    }
}

#[deriving(Show)]
pub struct Layer {
    name: String,
    opacity: f32,
    visible: bool,
    tiles: Vec<Vec<u32>>
}

impl Layer {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>, width: uint) -> Result<Layer, String> {
        let ((o, v), n) = get_attrs!(
            attrs,
            optionals: [("opacity", opacity, f32, |v:String| from_str(v[])),
                        ("visible", visible, bool, |v:String| from_str(v[]).map(|x:int| x == 1))],
            required: [("name", name, String, |v| Some(v))],
            "layer must have a name".to_string());
        let mut tiles = Vec::new();
        parse_tag!(parser, "layer",
                   "data" => |attrs| {
                        tiles = try!(parse_data(parser, attrs, width));
                        Ok(())
                   });
        Ok(Layer {name: n, opacity: o.unwrap_or(1.0), visible: v.unwrap_or(true), tiles: tiles })
    }
}

pub fn parse_data<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>, width: uint) -> Result<Vec<Vec<u32>>, String> {
    let ((), (e, c)) = get_attrs!(
        attrs,
        optionals: [],
        required: [("encoding", encoding, String, |v| Some(v)),
                   ("compression", compression, String, |v| Some(v))],
        "".to_string());
    if !(e[] == "base64" && c[] == "zlib") {
        return Err("Only base64 and zlib allowed for the moment".to_string());
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
                                Err(_) => return Err("Zlib decoding error".to_string())
                            }
                            if row.len() == width {
                                data.push(row);
                                row = Vec::new();
                            }
                        }
                    }
                    Err(e) => return Err(format!("{}", e))
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

pub fn parse<B: Buffer>(parser: &mut EventReader<B>) -> Result<Map, String>{
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
