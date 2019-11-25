

use std::fs::File;
use std::path::Path;
use tiled::parse;

fn main() {
    let file = File::open(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    println!("Opened file");
    let map = parse(file).unwrap();
    println!("{:?}", map);
    println!("{:?}", map.get_tileset_by_gid(22));
}
