#![feature(globs, macro_rules)]
extern crate xml;

use std::io::File;
use std::io::BufferedReader;

use xml::reader::EventReader;
use xml::common::Attribute;
use xml::reader::events::*;

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

            if !(true $(&& $oVar.is_some())*) {
                return Err($msg);
            }
            (($($oVar),*), ($($var.unwrap()),*))
        }
    }
}

macro_rules! parse_tag {
    ($parser:expr, $close_tag:expr, $open_tag:expr => $open_method:expr) => {
        loop {
            match $parser.next() {
                StartElement {name, attributes, ..} => {
                    if name.local_name[] == $open_tag {
                        $open_method(attributes);
                    }
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
    tile_height: int
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

        parse_tag!(parser, "map", 
                   "tileset" => |attrs| {
                        let t = try!(Tileset::new(parser, attrs));
                        println!("{}", t);
                        Ok(())
                   });
        Ok(Map {version: v, width: w, height: h, tile_width: tw, tile_height: th})
    }
}

#[deriving(Show)]
pub struct Tileset {
    first_gid: int,
    name: String,
}

impl Tileset {
    pub fn new<B: Buffer>(parser: &mut EventReader<B>, attrs: Vec<Attribute>) -> Result<Tileset, String> {
        let ((), (g, n)) = get_attrs!(
           attrs,
           optionals: [],
           required: [("firstgid", first_gid, int, |v:String| from_str(v[])),
                      ("name", name, String, |v| Some(v))],
           "tileset must have a firstgid and name with correct types".to_string());

        parse_tag!(parser, "tileset",
                   "image" => |attrs| {
                        let i = try!(Image::new(parser, attrs));
                        println!("{}", i);
                        Ok(())
                   });
        Ok(Tileset {first_gid: g, name: n})
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
        
        parse_tag!(parser, "image", "" => {});
        Ok(Image {source: s, width: w, height: h})
    }
}

pub fn parse<B: Buffer>(parser: &mut EventReader<B>) -> Result<(), String>{
    loop {
        match parser.next() {
            StartElement {name, attributes, ..}  => {
                if name.local_name[] == "map" {
                    let m = try!(Map::new(parser, attributes));
                    println!("{}", m);
                    return Ok(());
                }
            }
            _ => {}
        }
    }
}
