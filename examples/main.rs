use tiled::map::Map;

fn main() {
    let map = Map::parse_file("assets/tiled_base64_zlib.tmx").unwrap();
    println!("{:?}", map);
    println!("{:?}", map.tileset_by_gid(22));
}
