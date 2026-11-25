use std::{io::Cursor, path::Path};

use tiled::Loader;

#[test]
fn test_csv_whitespace_and_invalid_values() {
    for token in [
        "0",
        "1",
        "35",
        "2147483649",
        "4294967295",
        "+35",
        " 35 ",
        "\t35\r\n",
        "\u{2003}35\u{2003}",
        "",
        " ",
        "-1",
        "4294967296",
        "3x",
        "0 1",
        "١",
    ] {
        let xml = format!(
            "<map version=\"1.4\" orientation=\"orthogonal\" width=\"1\" height=\"1\" tilewidth=\"32\" tileheight=\"32\"><tileset firstgid=\"1\" name=\"tiles\" tilewidth=\"32\" tileheight=\"32\" tilecount=\"84\" columns=\"14\"><image source=\"tiles.png\" width=\"448\" height=\"192\"/></tileset><layer name=\"tiles\" width=\"1\" height=\"1\"><data encoding=\"csv\">{token}</data></layer></map>"
        );
        let mut loader =
            Loader::with_reader(|_: &Path| Ok::<_, std::io::Error>(Cursor::new(xml.as_bytes())));
        let result = loader.load_tmx_map("assets/generated.tmx");
        match token.trim().parse::<u32>() {
            Ok(bits) => {
                let map = result.unwrap();
                let layer = map.get_layer(0).unwrap().as_tile_layer().unwrap();
                let tile = layer.get_tile(0, 0);
                let gid = bits & 0x1fff_ffff;
                assert_eq!(tile.is_some(), gid != 0, "{token:?}");
                if let Some(tile) = tile {
                    assert_eq!(tile.id(), gid - 1);
                    assert_eq!(tile.flip_h, bits & 0x8000_0000 != 0);
                    assert_eq!(tile.flip_v, bits & 0x4000_0000 != 0);
                    assert_eq!(tile.flip_d, bits & 0x2000_0000 != 0);
                }
            }
            Err(expected) => match result {
                Err(tiled::Error::CsvDecodingError(
                    tiled::CsvDecodingError::TileDataParseError(actual),
                )) => assert_eq!(actual.kind(), expected.kind(), "{token:?}"),
                other => panic!("expected a CSV parse error for {token:?}, got {other:?}"),
            },
        }
    }
}
