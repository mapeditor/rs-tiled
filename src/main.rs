#![feature(globs)]

extern crate serialize;
extern crate xml;
extern crate tiled;

use serialize::base64::{FromBase64};
use std::io::File;
use std::io::BufferedReader;
use xml::reader::EventReader;
use xml::reader::events::*;
use tiled::parse;

fn main() {
    let file = File::open(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    let reader = BufferedReader::new(file);
    let mut parser = EventReader::new(reader);
    println!("{}", parse(&mut parser));
}
