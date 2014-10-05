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

        while true {
            match parser.next() {
                StartElement {name, attributes, ..} => {
                    if name.local_name[] == "tileset" {
                        let t = try!(Tileset::new(parser, attributes));
                        println!("{}", t);
                    }
                }
                EndElement {name, ..} => {
                    if name.local_name[] == "map" {
                        return Ok(Map {version: v, width: w, height: h, tile_width: tw, tile_height: th});
                    }
                }
                _ => {}
            }
        }
        Err("This should never happen".to_string())
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

       Ok(Tileset {first_gid: g, name: n})
   }
}

pub fn parse<B: Buffer>(parser: &mut EventReader<B>) -> Result<(), String>{
    while true {
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
    Ok(())
}
