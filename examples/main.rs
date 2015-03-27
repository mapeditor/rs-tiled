extern crate tiled;

use std::path::Path;
use std::fs::File;
use tiled::parse;

fn main() {
    let file = File::open(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    println!("Opened file");
    let map = parse(file).unwrap();
    println!("{:?}", map);
    println!("{:?}", map.get_tileset_by_gid(22));
}
