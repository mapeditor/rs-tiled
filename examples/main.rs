use std::fs::File;
use std::path::PathBuf;

use tiled::map::Map;

fn main() {
    let path = PathBuf::from("assets/tiled_base64_zlib.tmx");
    let file = File::open(&path).unwrap();
    println!("Opened file");
    let map = Map::parse_reader(file, Some(&path)).unwrap();
    println!("{:?}", map);
    println!("{:?}", map.tileset_by_gid(22));
}
