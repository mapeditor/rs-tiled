use std::path::PathBuf;
use tiled::{
    Color, FiniteTileLayer, GroupLayer, Layer, LayerType, Loader, Map, ObjectLayer, PropertyValue,
    ResourceCache, TileLayer,
};

fn as_tile_layer<'map>(layer: Layer<'map>) -> TileLayer<'map> {
    match layer.layer_type() {
        LayerType::TileLayer(x) => x,
        _ => panic!("Not a tile layer"),
    }
}

fn as_finite<'map>(data: TileLayer<'map>) -> FiniteTileLayer<'map> {
    match data {
        TileLayer::Finite(data) => data,
        TileLayer::Infinite(_) => panic!("Not a finite tile layer"),
    }
}

fn as_object_layer<'map>(layer: Layer<'map>) -> ObjectLayer<'map> {
    match layer.layer_type() {
        LayerType::ObjectLayer(x) => x,
        _ => panic!("Not an object layer"),
    }
}

fn as_group_layer<'map>(layer: Layer<'map>) -> GroupLayer<'map> {
    match layer.layer_type() {
        LayerType::GroupLayer(x) => x,
        _ => panic!("Not a group layer"),
    }
}

fn compare_everything_but_tileset_sources(r: &Map, e: &Map) {
    assert_eq!(r.version(), e.version());
    assert_eq!(r.orientation, e.orientation);
    assert_eq!(r.width, e.width);
    assert_eq!(r.height, e.height);
    assert_eq!(r.tile_width, e.tile_width);
    assert_eq!(r.tile_height, e.tile_height);
    assert_eq!(r.properties, e.properties);
    assert_eq!(r.background_color, e.background_color);
    assert_eq!(r.infinite(), e.infinite());
    // TODO: Also compare layers
    /*
    r.layers()
        .zip(e.layers())
        .for_each(|(r, e)| assert_eq!(r, e)); */
}

#[test]
fn test_gzip_and_zlib_encoded_and_raw_are_the_same() {
    let mut loader = Loader::new();
    let z = loader.load_tmx_map("assets/tiled_base64_zlib.tmx").unwrap();
    let g = loader.load_tmx_map("assets/tiled_base64_gzip.tmx").unwrap();
    let r = loader.load_tmx_map("assets/tiled_base64.tmx").unwrap();
    let zstd = loader
        .load_tmx_map("assets/tiled_base64_zstandard.tmx")
        .unwrap();
    let c = Loader::new().load_tmx_map("assets/tiled_csv.tmx").unwrap();
    compare_everything_but_tileset_sources(&z, &g);
    compare_everything_but_tileset_sources(&z, &r);
    compare_everything_but_tileset_sources(&z, &c);
    compare_everything_but_tileset_sources(&z, &zstd);

    let layer = as_finite(as_tile_layer(c.get_layer(0).unwrap()));
    {
        assert_eq!(layer.width(), 100);
        assert_eq!(layer.height(), 100);
    }

    assert_eq!(layer.get_tile(0, 0).unwrap().id(), 34);
    assert_eq!(layer.get_tile(0, 1).unwrap().id(), 16);
    assert!(layer.get_tile(0, 2).is_none());
    assert_eq!(layer.get_tile(1, 2).unwrap().id(), 16);
    assert!((0..99).map(|x| layer.get_tile(x, 99)).all(|t| t.is_none()));
}

#[test]
fn test_external_tileset() {
    let mut loader = Loader::new();

    let r = loader.load_tmx_map("assets/tiled_base64.tmx").unwrap();
    let e = loader
        .load_tmx_map("assets/tiled_base64_external.tmx")
        .unwrap();
    compare_everything_but_tileset_sources(&r, &e);
}

#[test]
fn test_sources() {
    let mut loader = Loader::new();
    let e = loader
        .load_tmx_map("assets/tiled_base64_external.tmx")
        .unwrap();
    assert_eq!(
        e.tilesets()[0],
        loader.cache().get_tileset("assets/tilesheet.tsx").unwrap()
    );
    assert_eq!(
        e.tilesets()[0].image.as_ref().unwrap().source,
        PathBuf::from("assets/tilesheet.png")
    );
}

#[test]
fn test_just_tileset() {
    let mut loader = Loader::new();
    let r = loader
        .load_tmx_map("assets/tiled_base64_external.tmx")
        .unwrap();
    assert_eq!(
        r.tilesets()[0],
        loader.cache().get_tileset("assets/tilesheet.tsx").unwrap()
    );
}

#[test]
fn test_infinite_map() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_base64_zlib_infinite.tmx")
        .unwrap();

    if let TileLayer::Infinite(inf) = &as_tile_layer(r.get_layer(1).unwrap()) {
        assert_eq!(inf.get_tile(2, 10).unwrap().id(), 5);
        assert_eq!(inf.get_tile(5, 36).unwrap().id(), 73);
        assert_eq!(inf.get_tile(15, 15).unwrap().id(), 22);
    } else {
        panic!("It is wrongly recognised as a finite map");
    }
    if let TileLayer::Infinite(inf) = &as_tile_layer(r.get_layer(0).unwrap()) {
        // NW corner
        assert_eq!(inf.get_tile(-16, 0).unwrap().id(), 17);
        assert!(inf.get_tile(-17, 0).is_none());
        assert!(inf.get_tile(-16, -1).is_none());

        // SW corner
        assert_eq!(inf.get_tile(-16, 47).unwrap().id(), 17);
        assert!(inf.get_tile(-17, 47).is_none());
        assert!(inf.get_tile(-16, 48).is_none());

        // NE corner
        assert_eq!(inf.get_tile(31, 0).unwrap().id(), 17);
        assert!(inf.get_tile(31, -1).is_none());
        assert!(inf.get_tile(32, 0).is_none());

        // SE corner
        assert_eq!(inf.get_tile(31, 47).unwrap().id(), 17);
        assert!(inf.get_tile(32, 47).is_none());
        assert!(inf.get_tile(31, 48).is_none());
    } else {
        panic!("It is wrongly recognised as a finite map");
    }
}

#[test]
fn test_image_layers() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_image_layers.tmx")
        .unwrap();
    assert_eq!(r.layers().len(), 2);
    let mut image_layers = r.layers().map(|layer| {
        if let LayerType::ImageLayer(img) = layer.layer_type() {
            (img, layer)
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
            .unwrap_or_else(|| panic!("{}'s image shouldn't be None", second.1.name));
        assert_eq!(image.source, PathBuf::from("assets/tilesheet.png"));
        assert_eq!(image.width, 448);
        assert_eq!(image.height, 192);
    }
}

#[test]
fn test_tile_property() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_base64.tmx")
        .unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) = r.tilesets()[0]
        .get_tile(1)
        .unwrap()
        .properties
        .get("a tile property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("123", prop_value);
}

#[test]
fn test_layer_property() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_base64.tmx")
        .unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.get_layer(0).unwrap().properties.get("prop3")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("Line 1\r\nLine 2\r\nLine 3,\r\n  etc\r\n   ", prop_value);
}

#[test]
fn test_object_group_property() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_object_groups.tmx")
        .unwrap();
    let group_layer = r.get_layer(1).unwrap();
    let group_layer = as_group_layer(group_layer);
    let sub_layer = group_layer.get_layer(0).unwrap();
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
    let r = Loader::new()
        .load_tmx_map("assets/tiled_base64.tmx")
        .unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.tilesets()[0].properties.get("tileset property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("tsp", prop_value);
}

#[test]
fn test_flipped() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_flipped.tmx")
        .unwrap();
    let layer = as_tile_layer(r.get_layer(0).unwrap());

    let t1 = layer.get_tile(0, 0).unwrap();
    let t2 = layer.get_tile(1, 0).unwrap();
    let t3 = layer.get_tile(0, 1).unwrap();
    let t4 = layer.get_tile(1, 1).unwrap();
    assert_eq!(t1.id(), t2.id());
    assert_eq!(t2.id(), t3.id());
    assert_eq!(t3.id(), t4.id());
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
}

#[test]
fn test_ldk_export() {
    let r = Loader::new()
        .load_tmx_map("assets/ldk_tiled_export.tmx")
        .unwrap();
    let layer = as_finite(as_tile_layer(r.get_layer(0).unwrap()));
    {
        assert_eq!(layer.width(), 8);
        assert_eq!(layer.height(), 8);
    }
    assert!(layer.get_tile(0, 0).is_none());
    assert_eq!(layer.get_tile(0, 1).unwrap().id(), 0);
}

#[test]
fn test_parallax_layers() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_parallax.tmx")
        .unwrap();
    for (i, layer) in r.layers().enumerate() {
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
    let r = Loader::new()
        .load_tmx_map("assets/tiled_object_property.tmx")
        .unwrap();
    let layer = r.get_layer(1).unwrap();
    let prop_value = if let Some(PropertyValue::ObjectValue(v)) = as_object_layer(layer)
        .get_object(0)
        .unwrap()
        .properties
        .get("object property")
    {
        *v
    } else {
        0
    };
    assert_eq!(3, prop_value);
}

#[test]
fn test_tint_color() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_image_layers.tmx")
        .unwrap();
    assert_eq!(
        r.get_layer(0).unwrap().tint_color,
        Some(Color {
            alpha: 0x12,
            red: 0x34,
            green: 0x56,
            blue: 0x78
        })
    );
    assert_eq!(
        r.get_layer(1).unwrap().tint_color,
        Some(Color {
            alpha: 0xFF,
            red: 0x12,
            green: 0x34,
            blue: 0x56
        })
    );
}

#[test]
fn test_group_layers() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_group_layers.tmx")
        .unwrap();

    // Depth = 0
    let layer_tile_1 = r.get_layer(0).unwrap();
    let layer_group_1 = r.get_layer(1).unwrap();
    let layer_group_2 = r.get_layer(2).unwrap();

    assert_eq!(
        Some(&PropertyValue::StringValue("value1".to_string())),
        layer_tile_1.properties.get("key")
    );
    assert_eq!(
        Some(&PropertyValue::ColorValue(Color {
            alpha: 0x12,
            red: 0x34,
            green: 0x56,
            blue: 0x78
        })),
        layer_group_1.properties.get("key")
    );
    assert_eq!(
        Some(&PropertyValue::StringValue("value5".to_string())),
        layer_group_2.properties.get("key")
    );

    // Depth = 1
    let layer_group_1 = as_group_layer(layer_group_1);
    let layer_tile_2 = layer_group_1.get_layer(0).unwrap();
    let layer_group_2 = as_group_layer(layer_group_2);
    let layer_group_3 = layer_group_2.get_layer(0).unwrap();
    assert_eq!(
        Some(&PropertyValue::StringValue("value2".to_string())),
        layer_tile_2.properties.get("key")
    );
    assert_eq!(
        Some(&PropertyValue::StringValue("value6".to_string())),
        layer_group_3.properties.get("key")
    );

    // Depth = 2
    let layer_group_3 = as_group_layer(layer_group_3);
    let layer_tile_3 = layer_group_3.get_layer(0).unwrap();
    assert_eq!(
        Some(&PropertyValue::StringValue("value3".to_string())),
        layer_tile_3.properties.get("key")
    );
}
