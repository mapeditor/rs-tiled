use std::{collections::HashMap, io::Cursor, path::Path};

use tiled::{Loader, Map};

fn load_chunks(data: &str) -> tiled::Result<Map> {
    let xml = format!(
        "<map version=\"1.4\" orientation=\"orthogonal\" width=\"0\" height=\"0\" tilewidth=\"32\" tileheight=\"32\" infinite=\"1\"><tileset firstgid=\"1\" name=\"tiles\" tilewidth=\"32\" tileheight=\"32\" tilecount=\"84\" columns=\"14\"><image source=\"tiles.png\" width=\"448\" height=\"192\"/></tileset><layer name=\"tiles\" width=\"0\" height=\"0\"><data encoding=\"csv\">{data}</data></layer></map>"
    );
    Loader::with_reader(|_: &Path| Ok::<_, std::io::Error>(Cursor::new(xml.as_bytes())))
        .load_tmx_map("assets/generated.tmx")
}

#[test]
fn test_unaligned_overlapping_chunks() {
    let mut expected = HashMap::new();
    let mut data = String::new();
    for (index, (x, y, width, height)) in [(-19, -17, 37, 35), (3, 2, 19, 18), (15, -3, 5, 30)]
        .into_iter()
        .enumerate()
    {
        let mut values = Vec::new();
        for dy in 0..height {
            for dx in 0..width {
                let i = (dy * width + dx) as u32 + index as u32;
                let gid = i % 85;
                let bits = if gid == 0 { 0 } else { gid | ((i % 8) << 29) };
                expected.insert((x + dx, y + dy), bits);
                values.push(bits.to_string());
            }
        }
        data.push_str(&format!(
            "<chunk x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\">{}</chunk>",
            values.join(",")
        ));
    }
    let map = load_chunks(&data).unwrap();
    let layer = map.get_layer(0).unwrap().as_tile_layer().unwrap();
    for y in -40..60 {
        for x in -40..60 {
            let actual = layer.get_tile(x, y).map_or(0, |tile| {
                (tile.id() + 1)
                    | ((tile.flip_h as u32) << 31)
                    | ((tile.flip_v as u32) << 30)
                    | ((tile.flip_d as u32) << 29)
            });
            assert_eq!(
                actual,
                expected.get(&(x, y)).copied().unwrap_or(0),
                "({x}, {y})"
            );
        }
    }
}

#[test]
fn test_incomplete_chunks_are_rejected() {
    for data in [
        "<chunk x=\"-3\" y=\"-1\" width=\"17\" height=\"2\">1,2,3</chunk>",
        "<chunk x=\"0\" y=\"0\" width=\"16\" height=\"1\">1</chunk>",
    ] {
        assert!(matches!(
            load_chunks(data),
            Err(tiled::Error::InvalidTileFound)
        ));
    }
}
