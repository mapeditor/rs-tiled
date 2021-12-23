use std::fs::File;
use std::path::Path;
use tiled::{
    error::TiledError, layers::LayerData, map::Map, properties::PropertyValue, tileset::Tileset,
};

fn parse_map_without_source(p: &Path) -> Result<Map, TiledError> {
    let file = File::open(p).unwrap();
    return Map::parse_reader(file, None);
}

#[test]
fn test_gzip_and_zlib_encoded_and_raw_are_the_same() {
    let z = parse_map_without_source(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    let g = parse_map_without_source(&Path::new("assets/tiled_base64_gzip.tmx")).unwrap();
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let zstd = parse_map_without_source(&Path::new("assets/tiled_base64_zstandard.tmx")).unwrap();
    let c = parse_map_without_source(&Path::new("assets/tiled_csv.tmx")).unwrap();
    assert_eq!(z, g);
    assert_eq!(z, r);
    assert_eq!(z, c);
    assert_eq!(z, zstd);

    if let LayerData::Finite(tiles) = &c.layers[0].tiles {
        assert_eq!(tiles.len(), 100);
        assert_eq!(tiles[0].len(), 100);
        assert_eq!(tiles[99].len(), 100);
        assert_eq!(tiles[0][0].gid, 35);
        assert_eq!(tiles[1][0].gid, 17);
        assert_eq!(tiles[2][0].gid, 0);
        assert_eq!(tiles[2][1].gid, 17);
        assert!(tiles[99].iter().map(|t| t.gid).all(|g| g == 0));
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_external_tileset() {
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let e = Map::parse_file(&Path::new("assets/tiled_base64_external.tmx")).unwrap();
    // Compare everything BUT source
    assert_eq!(r.version, e.version);
    assert_eq!(r.orientation, e.orientation);
    assert_eq!(r.width, e.width);
    assert_eq!(r.height, e.height);
    assert_eq!(r.tile_width, e.tile_width);
    assert_eq!(r.tile_height, e.tile_height);
    assert_eq!(r.tilesets, e.tilesets);
    assert_eq!(r.layers, e.layers);
    assert_eq!(r.image_layers, e.image_layers);
    assert_eq!(r.object_groups, e.object_groups);
    assert_eq!(r.properties, e.properties);
    assert_eq!(r.background_color, e.background_color);
    assert_eq!(r.infinite, e.infinite);
}

#[test]
fn test_just_tileset() {
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let t = Tileset::parse(File::open(Path::new("assets/tilesheet.tsx")).unwrap(), 1).unwrap();
    assert_eq!(r.tilesets[0], t);
}

#[test]
fn test_infinite_tileset() {
    let r = Map::parse_file(&Path::new("assets/tiled_base64_zlib_infinite.tmx")).unwrap();

    if let LayerData::Infinite(chunks) = &r.layers[0].tiles {
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
    let r = parse_map_without_source(&Path::new("assets/tiled_image_layers.tmx")).unwrap();
    assert_eq!(r.image_layers.len(), 2);
    {
        let first = &r.image_layers[0];
        assert_eq!(first.name, "Image Layer 1");
        assert!(
            first.image.is_none(),
            "{}'s image should be None",
            first.name
        );
    }
    {
        let second = &r.image_layers[1];
        assert_eq!(second.name, "Image Layer 2");
        let image = second
            .image
            .as_ref()
            .expect(&format!("{}'s image shouldn't be None", second.name));
        assert_eq!(image.source, "tilesheet.png");
        assert_eq!(image.width, 448);
        assert_eq!(image.height, 192);
    }
}

#[test]
fn test_tile_property() {
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
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
fn test_object_group_property() {
    let r = parse_map_without_source(&Path::new("assets/tiled_object_groups.tmx")).unwrap();
    let prop_value: bool = if let Some(&PropertyValue::BoolValue(ref v)) = r.object_groups[0]
        .properties
        .get("an object group property")
    {
        *v
    } else {
        false
    };
    assert!(prop_value);
}
#[test]
fn test_tileset_property() {
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
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
    let r = Map::parse_file(&Path::new("assets/tiled_flipped.tmx")).unwrap();

    if let LayerData::Finite(tiles) = &r.layers[0].tiles {
        let t1 = tiles[0][0];
        let t2 = tiles[0][1];
        let t3 = tiles[1][0];
        let t4 = tiles[1][1];
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
    let r = Map::parse_file(&Path::new("assets/ldk_tiled_export.tmx")).unwrap();
    if let LayerData::Finite(tiles) = &r.layers[0].tiles {
        assert_eq!(tiles.len(), 8);
        assert_eq!(tiles[0].len(), 8);
        assert_eq!(tiles[0][0].gid, 0);
        assert_eq!(tiles[1][0].gid, 1);
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}
