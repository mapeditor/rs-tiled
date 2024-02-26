use std::path::PathBuf;
use tiled::{
    Color, FiniteTileLayer, GroupLayer, HorizontalAlignment, Layer, LayerType, Loader, Map,
    ObjectLayer, ObjectShape, PropertyValue, ResourceCache, TileLayer, TilesetLocation,
    VerticalAlignment, WangId,
};

fn as_finite<'map>(data: TileLayer<'map>) -> FiniteTileLayer<'map> {
    match data {
        TileLayer::Finite(data) => data,
        TileLayer::Infinite(_) => panic!("Not a finite tile layer"),
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

    let layer = as_finite(c.get_layer(0).unwrap().as_tile_layer().unwrap());
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

    if let TileLayer::Infinite(inf) = &r.get_layer(1).unwrap().as_tile_layer().unwrap() {
        assert_eq!(inf.get_tile(2, 10).unwrap().id(), 5);
        assert_eq!(inf.get_tile(5, 36).unwrap().id(), 73);
        assert_eq!(inf.get_tile(15, 15).unwrap().id(), 22);
    } else {
        panic!("It is wrongly recognised as a finite map");
    }
    if let TileLayer::Infinite(inf) = &r.get_layer(0).unwrap().as_tile_layer().unwrap() {
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
        if let LayerType::Image(img) = layer.layer_type() {
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
    let group_layer = group_layer.as_group_layer().unwrap();
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
    let layer = r.get_layer(0).unwrap().as_tile_layer().unwrap();

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
    let layer = as_finite(r.get_layer(0).unwrap().as_tile_layer().unwrap());
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
    let prop_value = if let Some(PropertyValue::ObjectValue(v)) = layer
        .as_object_layer()
        .unwrap()
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
fn test_class_property() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_class_property.tmx")
        .unwrap();
    let layer = r.get_layer(1).unwrap();
    if let Some(PropertyValue::ClassValue {
        property_type,
        properties,
    }) = layer
        .as_object_layer()
        .unwrap()
        .get_object(0)
        .unwrap()
        .properties
        .get("class property")
    {
        assert_eq!(property_type, "test_type");
        assert_eq!(
            properties.get("test_property_1").unwrap(),
            &PropertyValue::IntValue(3)
        );
    } else {
        panic!("Expected class property");
    };
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
            blue: 0x78,
        })
    );
    assert_eq!(
        r.get_layer(1).unwrap().tint_color,
        Some(Color {
            alpha: 0xFF,
            red: 0x12,
            green: 0x34,
            blue: 0x56,
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
            blue: 0x78,
        })),
        layer_group_1.properties.get("key")
    );
    assert_eq!(
        Some(&PropertyValue::StringValue("value5".to_string())),
        layer_group_2.properties.get("key")
    );

    // Depth = 1
    let layer_group_1 = layer_group_1.as_group_layer().unwrap();
    let layer_tile_2 = layer_group_1.get_layer(0).unwrap();
    let layer_group_2 = layer_group_2.as_group_layer().unwrap();
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
    let layer_group_3 = layer_group_3.as_group_layer().unwrap();
    let layer_tile_3 = layer_group_3.get_layer(0).unwrap();
    assert_eq!(
        Some(&PropertyValue::StringValue("value3".to_string())),
        layer_tile_3.properties.get("key")
    );
}

#[test]
fn test_object_template_property() {
    let r = Loader::new()
        .load_tmx_map("assets/tiled_object_template.tmx")
        .unwrap();

    let object_layer = r.get_layer(1).unwrap().as_object_layer().unwrap();
    let object = object_layer.get_object(0).unwrap(); // The templated object
    let object_nt = object_layer.get_object(1).unwrap(); // The non-templated object

    // Test core properties
    assert_eq!(
        object.shape,
        ObjectShape::Rect {
            width: 32.0,
            height: 32.0,
        }
    );
    assert_eq!(object.x, 32.0);
    assert_eq!(object.y, 32.0);

    // Test properties are copied over
    assert_eq!(
        Some(&PropertyValue::IntValue(1)),
        object.properties.get("property")
    );

    // Test tileset handling
    assert_eq!(
        object.get_tile().unwrap().get_tileset().name,
        "tilesheet_template"
    );
    assert_eq!(
        object_nt.get_tile().unwrap().get_tileset().name,
        "tilesheet"
    );
    assert!(matches!(
        object.get_tile().unwrap().tileset_location(),
        TilesetLocation::Template(..)
    ));
    assert_eq!(
        object_nt.get_tile().unwrap().tileset_location(),
        &TilesetLocation::Map(0)
    );
    assert_eq!(object.get_tile().unwrap().id(), 44);
    assert_eq!(object_nt.get_tile().unwrap().id(), 44);
}

#[test]
fn test_templates() {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/templates/example.tmx").unwrap();

    assert_eq!(loader.cache().templates.len(), 3);
    assert_eq!(
        if let LayerType::Tiles(x) = map.get_layer(0).unwrap().layer_type() {
            x
        } else {
            panic!()
        }
        .get_tile(0, 0)
        .unwrap()
        .get_tileset()
        .image
        .as_ref()
        .unwrap()
        .source
        .canonicalize()
        .unwrap(),
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/tilesheet.png"))
            .canonicalize()
            .unwrap()
    );
}

#[test]
fn test_reading_wang_sets() {
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map("assets/tiled_csv_wangsets.tmx")
        .unwrap();

    // We will pick some random data from the wangsets for tessting
    let tileset = map.tilesets().get(0).unwrap();
    assert_eq!(tileset.wang_sets.len(), 3);
    let wangset_2 = tileset.wang_sets.get(1).unwrap();
    let tile_10 = wangset_2.wang_tiles.get(&10).unwrap();
    assert_eq!(tile_10.wang_id, WangId([2u8, 2, 0, 2, 0, 2, 2, 2]));
    let wangset_3 = tileset.wang_sets.get(2).unwrap();
    let color_2 = wangset_3.wang_colors.get(1).unwrap();
    let readed_damage = color_2.properties.get("Damage").unwrap();
    let damage_value = &PropertyValue::FloatValue(32.1);
    assert_eq!(readed_damage, damage_value);
}

#[test]
fn test_text_object() {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/tiled_text_object.tmx").unwrap();

    let group = map.get_layer(0).unwrap().as_object_layer().unwrap();
    match &group.objects().next().unwrap().shape {
        ObjectShape::Text {
            font_family,
            pixel_size,
            wrap,
            color,
            bold,
            italic,
            underline,
            strikeout,
            kerning,
            halign,
            valign,
            text,
            width,
            height,
        } => {
            assert_eq!(font_family.as_str(), "sans-serif");
            assert_eq!(*pixel_size, 16);
            assert_eq!(*wrap, false);
            assert_eq!(
                *color,
                Color {
                    red: 85,
                    green: 255,
                    blue: 127,
                    alpha: 100
                }
            );
            assert_eq!(*bold, true);
            assert_eq!(*italic, true);
            assert_eq!(*underline, true);
            assert_eq!(*strikeout, true);
            assert_eq!(*kerning, true);
            assert_eq!(*halign, HorizontalAlignment::Center);
            assert_eq!(*valign, VerticalAlignment::Bottom);
            assert_eq!(text.as_str(), "Test");
            assert_eq!(*width, 87.7188);
            assert_eq!(*height, 21.7969);
        }
        _ => panic!(),
    };
}
