use std::path::Path;
use std::{fs::File, path::PathBuf};
use tiled::{LayerData, Map, PropertyValue, TiledError, Tileset};
use tiled::{LayerType, ObjectLayer, TileLayer};

fn as_tile_layer(layer: &LayerType) -> &TileLayer {
    match layer {
        LayerType::TileLayer(x) => x,
        _ => panic!("Not a tile layer"),
    }
}

fn as_object_layer(layer: &LayerType) -> &ObjectLayer {
    match layer {
        LayerType::ObjectLayer(x) => x,
        _ => panic!("Not an object layer"),
    }
}

fn parse_map_without_source(p: impl AsRef<Path>) -> Result<Map, TiledError> {
    let file = File::open(p).unwrap();
    return Map::parse_reader(file, None);
}

#[test]
fn test_gzip_and_zlib_encoded_and_raw_are_the_same() {
    let z = Map::parse_file("assets/tiled_base64_zlib.tmx").unwrap();
    let g = Map::parse_file("assets/tiled_base64_gzip.tmx").unwrap();
    let r = Map::parse_file("assets/tiled_base64.tmx").unwrap();
    let zstd = Map::parse_file("assets/tiled_base64_zstandard.tmx").unwrap();
    let c = Map::parse_file("assets/tiled_csv.tmx").unwrap();
    assert_eq!(z, g);
    assert_eq!(z, r);
    assert_eq!(z, c);
    assert_eq!(z, zstd);

    if let LayerData::Finite(tiles) = &as_tile_layer(&c.layers[0].layer_type).tiles {
        assert_eq!(tiles.len(), 100 * 100);
        assert_eq!(tiles[0].gid, 35);
        assert_eq!(tiles[100].gid, 17);
        assert_eq!(tiles[200].gid, 0);
        assert_eq!(tiles[200 + 1].gid, 17);
        assert!(tiles[9900..9999].iter().map(|t| t.gid).all(|g| g == 0));
    } else {
        panic!("It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_external_tileset() {
    let r = Map::parse_file("assets/tiled_base64.tmx").unwrap();
    let mut e = Map::parse_file("assets/tiled_base64_external.tmx").unwrap();
    e.tilesets[0].source = None;
    assert_eq!(r, e);
}

#[test]
fn test_sources() {
    let e = Map::parse_file("assets/tiled_base64_external.tmx").unwrap();
    assert_eq!(
        e.tilesets[0].source,
        Some(PathBuf::from("assets/tilesheet.tsx"))
    );
    assert_eq!(
        e.tilesets[0].image.as_ref().unwrap().source,
        PathBuf::from("assets/tilesheet.png")
    );
}

#[test]
fn test_just_tileset() {
    let r = Map::parse_file("assets/tiled_base64_external.tmx").unwrap();
    let path = "assets/tilesheet.tsx";
    let t = Tileset::parse_with_path(File::open(path).unwrap(), 1, path).unwrap();
    assert_eq!(r.tilesets[0], t);
}

#[test]
fn test_infinite_tileset() {
    let r = Map::parse_file("assets/tiled_base64_zlib_infinite.tmx").unwrap();

    if let LayerData::Infinite(chunks) = &as_tile_layer(&r.layers[0].layer_type).tiles {
        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[&(0, 0)].width, 32);
        assert_eq!(chunks[&(0, 0)].height, 32);
        assert_eq!(chunks[&(-32, 0)].width, 32);
        assert_eq!(chunks[&(0, 32)].height, 32);
        assert_eq!(chunks[&(-32, 32)].height, 32);
    } else {
        assert!(false, "It is wrongly recognised as a finite map");
    }
}

#[test]
fn test_image_layers() {
    let r = Map::parse_file("assets/tiled_image_layers.tmx").unwrap();
    assert_eq!(r.layers.len(), 2);
    let mut image_layers = r.layers.iter().map(|x| {
        if let LayerType::ImageLayer(img) = &x.layer_type {
            (img, x)
        } else {
            panic!("Found layer that isn't an image layer")
        }
    });
    {
        let first = image_layers.next().unwrap();
        assert_eq!(first.1.name, "Image Layer 1");
        assert!(
            first.0.image.is_none(),
            "{}'s image should be None",
            first.1.name
        );
    }
    {
        let second = image_layers.next().unwrap();
        assert_eq!(second.1.name, "Image Layer 2");
        let image = second
            .0
            .image
            .as_ref()
            .expect(&format!("{}'s image shouldn't be None", second.1.name));
        assert_eq!(image.source, PathBuf::from("assets/tilesheet.png"));
        assert_eq!(image.width, 448);
        assert_eq!(image.height, 192);
    }
}

#[test]
fn test_tile_property() {
    let r = Map::parse_file("assets/tiled_base64.tmx").unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.tilesets[0].tiles[0].properties.get("a tile property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("123", prop_value);
}

#[test]
fn test_layer_property() {
    let r = Map::parse_file(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let prop_value: String =
        if let Some(&PropertyValue::StringValue(ref v)) = r.layers[0].properties.get("prop3") {
            v.clone()
        } else {
            String::new()
        };
    assert_eq!("Line 1\r\nLine 2\r\nLine 3,\r\n  etc\r\n   ", prop_value);
}

#[test]
fn test_object_group_property() {
    let r = Map::parse_file("assets/tiled_object_groups.tmx").unwrap();
    let sub_layer = match r.layers[1].layer_type {
        LayerType::GroupLayer(ref layer) => &layer.layers[0],
        _ => { panic!("Layer was expected to be a group layer"); }
    };
    let prop_value: bool = if let Some(&PropertyValue::BoolValue(ref v)) =
        sub_layer.properties.get("an object group property")
    {
        *v
    } else {
        false
    };
    assert!(prop_value);
}
#[test]
fn test_tileset_property() {
    let r = Map::parse_file("assets/tiled_base64.tmx").unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.tilesets[0].properties.get("tileset property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("tsp", prop_value);
}

#[test]
fn test_flipped_gid() {
    let r = Map::parse_file("assets/tiled_flipped.tmx").unwrap();

    if let LayerData::Finite(tiles) = &as_tile_layer(&r.layers[0].layer_type).tiles {
        let t1 = tiles[0];
        let t2 = tiles[1];
        let t3 = tiles[2];
        let t4 = tiles[3];
        assert_eq!(t1.gid, t2.gid);
        assert_eq!(t2.gid, t3.gid);
        assert_eq!(t3.gid, t4.gid);
        assert!(t1.flip_d);
        assert!(t1.flip_h);
        assert!(t1.flip_v);
        assert!(!t2.flip_d);
        assert!(!t2.flip_h);
        assert!(t2.flip_v);
        assert!(!t3.flip_d);
        assert!(t3.flip_h);
        assert!(!t3.flip_v);
        assert!(t4.flip_d);
        assert!(!t4.flip_h);
        assert!(!t4.flip_v);
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_ldk_export() {
    let r = Map::parse_file("assets/ldk_tiled_export.tmx").unwrap();
    if let LayerData::Finite(tiles) = &as_tile_layer(&r.layers[0].layer_type).tiles {
        assert_eq!(tiles.len(), 8 * 8);
        assert_eq!(tiles[0].gid, 0);
        assert_eq!(tiles[8].gid, 1);
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_parallax_layers() {
    let r = Map::parse_file("assets/tiled_parallax.tmx").unwrap();
    for (i, layer) in r.layers.iter().enumerate() {
        match i {
            0 => {
                assert_eq!(layer.name, "Background");
                assert_eq!(layer.parallax_x, 0.5);
                assert_eq!(layer.parallax_y, 0.75);
            }
            1 => {
                assert_eq!(layer.name, "Middle");
                assert_eq!(layer.parallax_x, 1.0);
                assert_eq!(layer.parallax_y, 1.0);
            }
            2 => {
                assert_eq!(layer.name, "Foreground");
                assert_eq!(layer.parallax_x, 2.0);
                assert_eq!(layer.parallax_y, 2.0);
            }
            _ => panic!("unexpected layer"),
        }
    }
}

#[test]
fn test_object_property() {
    let r = parse_map_without_source(&Path::new("assets/tiled_object_property.tmx")).unwrap();
    let prop_value = if let Some(PropertyValue::ObjectValue(v)) =
        as_object_layer(&r.layers[1].layer_type).objects[0]
            .properties
            .get("object property")
    {
        *v
    } else {
        0
    };
    assert_eq!(3, prop_value);
}
