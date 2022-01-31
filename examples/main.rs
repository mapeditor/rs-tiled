use tiled::Map;

fn main() {
    let map = Map::parse_file("assets/tiled_base64_zlib.tmx").unwrap();
    println!("{:?}", map);
    println!("{:?}", map.get_tile(0, 0, 0));
}
