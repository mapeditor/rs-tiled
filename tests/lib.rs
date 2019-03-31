#[macro_use]
extern crate downcast_rs;
extern crate tiled;

use std::fs::File;
use std::path::Path;
use tiled::{
    parse, parse_file, parse_tileset, Group, Map, ObjectGroup, PropertyValue, TileLayer, TiledError,
};

fn read_from_file(p: &Path) -> Result<Map, TiledError> {
    let file = File::open(p).unwrap();
    return parse(file);
}

fn read_from_file_with_path(p: &Path) -> Result<Map, TiledError> {
    return parse_file(p);
}

#[test]
fn test_gzip_and_zlib_encoded_and_raw_are_the_same() {
    let z = read_from_file(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    let g = read_from_file(&Path::new("assets/tiled_base64_gzip.tmx")).unwrap();
    let r = read_from_file(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let c = read_from_file(&Path::new("assets/tiled_csv.tmx")).unwrap();
    assert_eq!(z, g);
    assert_eq!(z, r);
    assert_eq!(z, c);
}

#[test]
fn test_external_tileset() {
    let r = read_from_file(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let e = read_from_file_with_path(&Path::new("assets/tiled_base64_external.tmx")).unwrap();
    assert_eq!(r, e);
}

#[test]
fn test_just_tileset() {
    let r = read_from_file(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let t = parse_tileset(File::open(Path::new("assets/tilesheet.tsx")).unwrap(), 1).unwrap();
    assert_eq!(r.tilesets[0], t);
}

#[test]
fn test_image_layers() {
    let r = read_from_file(&Path::new("assets/tiled_image_layers.tmx")).unwrap();
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
    let r = read_from_file(&Path::new("assets/tiled_base64.tmx")).unwrap();
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
fn test_nested_groups() {
    let r = read_from_file(&Path::new("assets/tiled_group_csv.tmx")).unwrap();
    assert_eq!(r.layers.len(), 1);
    let group = r.groups.get(0).unwrap();
    assert_eq!(group.children.len(), 3);

    let second_group = group.children.get(0).unwrap().downcast_ref::<Group>();
    assert!(second_group.is_some());
    let second_group = second_group.unwrap();
    assert_eq!(second_group.children.len(), 1);
    assert_eq!(second_group.offset_x, 20.0);
    assert_eq!(second_group.offset_y, 22.0);

    let object_layer = group.children.get(1).unwrap().downcast_ref::<ObjectGroup>();
    assert!(object_layer.is_some());

    let tile_layer = group.children.get(2).unwrap().downcast_ref::<TileLayer>();
    assert!(tile_layer.is_some());
}
