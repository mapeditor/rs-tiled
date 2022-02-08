use std::path::PathBuf;
use tiled::{
    DefaultResourceCache, FiniteTileLayerData, Layer, LayerDataType, LayerType, MapTilesetType,
    ObjectLayer, ResourceCache, TileLayer, TileLayerData,
};
use tiled::{Map, PropertyValue};

fn as_tile_layer<'map>(layer: Layer<'map>) -> TileLayer<'map> {
    match layer.layer_type() {
        LayerType::TileLayer(x) => x,
        _ => panic!("Not a tile layer"),
    }
}

fn as_finite(data: &TileLayerData) -> &FiniteTileLayerData {
    match data {
        TileLayerData::Finite(data) => data,
        TileLayerData::Infinite(_) => panic!("Not a finite tile layer"),
    }
}

fn as_object_layer<'map>(layer: Layer<'map>) -> ObjectLayer<'map> {
    match layer.layer_type() {
        LayerType::ObjectLayer(x) => x,
        _ => panic!("Not an object layer"),
    }
}

fn compare_everything_but_tileset_sources(r: &Map, e: &Map) {
    assert_eq!(r.version, e.version);
    assert_eq!(r.orientation, e.orientation);
    assert_eq!(r.width, e.width);
    assert_eq!(r.height, e.height);
    assert_eq!(r.tile_width, e.tile_width);
    assert_eq!(r.tile_height, e.tile_height);
    assert_eq!(r.properties, e.properties);
    assert_eq!(r.background_color, e.background_color);
    assert_eq!(r.infinite, e.infinite);
    r.layers()
        .zip(e.layers())
        .for_each(|(r, e)| assert_eq!(r.data(), e.data()));
}

#[test]
fn test_gzip_and_zlib_encoded_and_raw_are_the_same() {
    let mut cache = DefaultResourceCache::new();
    let z = Map::parse_file("assets/tiled_base64_zlib.tmx", &mut cache).unwrap();
    let g = Map::parse_file("assets/tiled_base64_gzip.tmx", &mut cache).unwrap();
    let r = Map::parse_file("assets/tiled_base64.tmx", &mut cache).unwrap();
    let zstd = Map::parse_file("assets/tiled_base64_zstandard.tmx", &mut cache).unwrap();
    let c = Map::parse_file("assets/tiled_csv.tmx", &mut cache).unwrap();
    compare_everything_but_tileset_sources(&z, &g);
    compare_everything_but_tileset_sources(&z, &r);
    compare_everything_but_tileset_sources(&z, &c);
    compare_everything_but_tileset_sources(&z, &zstd);

    let layer = as_tile_layer(c.get_layer(0).unwrap());
    {
        let data = as_finite(layer.data());
        assert_eq!(data.width(), 100);
        assert_eq!(data.height(), 100);
    }

    assert_eq!(layer.get_tile(0, 0).unwrap().id, 34);
    assert_eq!(layer.get_tile(0, 1).unwrap().id, 16);
    assert!(layer.get_tile(0, 2).is_none());
    assert_eq!(layer.get_tile(1, 2).unwrap().id, 16);
    assert!((0..99).map(|x| layer.get_tile(x, 99)).all(|t| t.is_none()));
}

#[test]
fn test_external_tileset() {
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_base64.tmx", &mut cache).unwrap();
    let e = Map::parse_file("assets/tiled_base64_external.tmx", &mut cache).unwrap();
    compare_everything_but_tileset_sources(&r, &e);
}

#[test]
fn test_sources() {
    let mut cache = DefaultResourceCache::new();

    let e = Map::parse_file("assets/tiled_base64_external.tmx", &mut cache).unwrap();
    assert_eq!(
        *e.tilesets()[0].tileset_type(),
        MapTilesetType::External {
            path: PathBuf::from("assets/tilesheet.tsx"),
            tileset: cache.get_tileset("assets/tilesheet.tsx").unwrap()
        }
    );
    assert_eq!(
        e.tilesets()[0].tileset().image.as_ref().unwrap().source,
        PathBuf::from("assets/tilesheet.png")
    );
}

#[test]
fn test_just_tileset() {
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_base64_external.tmx", &mut cache).unwrap();
    assert_eq!(
        *r.tilesets()[0].tileset_type(),
        MapTilesetType::External {
            path: PathBuf::from("assets/tilesheet.tsx"),
            tileset: cache.get_tileset("assets/tilesheet.tsx").unwrap()
        }
    );
}

#[test]
fn test_infinite_tileset() {
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_base64_zlib_infinite.tmx", &mut cache).unwrap();

    if let TileLayerData::Infinite(inf) = &as_tile_layer(r.get_layer(0).unwrap()).data() {
        let chunks = &inf.chunks;
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
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_image_layers.tmx", &mut cache).unwrap();
    assert_eq!(r.layers().len(), 2);
    let mut image_layers = r.layers().map(|layer| layer.data()).map(|layer| {
        if let LayerDataType::ImageLayer(img) = &layer.layer_type {
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
            .expect(&format!("{}'s image shouldn't be None", second.1.name));
        assert_eq!(image.source, PathBuf::from("assets/tilesheet.png"));
        assert_eq!(image.width, 448);
        assert_eq!(image.height, 192);
    }
}

#[test]
fn test_tile_property() {
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_base64.tmx", &mut cache).unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) = r.tilesets()[0]
        .tileset()
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
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_base64.tmx", &mut cache).unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.get_layer(0).unwrap().data().properties.get("prop3")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("Line 1\r\nLine 2\r\nLine 3,\r\n  etc\r\n   ", prop_value);
}

#[test]
fn test_object_group_property() {
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_object_groups.tmx", &mut cache).unwrap();
    let prop_value: bool = if let Some(&PropertyValue::BoolValue(ref v)) = r
        .layers()
        .nth(1)
        .unwrap()
        .data()
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
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_base64.tmx", &mut cache).unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) = r.tilesets()[0]
        .tileset()
        .properties
        .get("tileset property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("tsp", prop_value);
}

#[test]
fn test_flipped() {
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_flipped.tmx", &mut cache).unwrap();
    let layer = as_tile_layer(r.get_layer(0).unwrap());

    let t1 = layer.get_tile(0, 0).unwrap();
    let t2 = layer.get_tile(1, 0).unwrap();
    let t3 = layer.get_tile(0, 1).unwrap();
    let t4 = layer.get_tile(1, 1).unwrap();
    assert_eq!(t1.id, t2.id);
    assert_eq!(t2.id, t3.id);
    assert_eq!(t3.id, t4.id);
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
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/ldk_tiled_export.tmx", &mut cache).unwrap();
    let layer = as_tile_layer(r.get_layer(0).unwrap());
    {
        let data = as_finite(layer.data());
        assert_eq!(data.width(), 8);
        assert_eq!(data.height(), 8);
    }
    assert!(layer.get_tile(0, 0).is_none());
    assert_eq!(layer.get_tile(0, 1).unwrap().id, 0);
}

#[test]
fn test_parallax_layers() {
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_parallax.tmx", &mut cache).unwrap();
    for (i, layer) in r.layers().enumerate() {
        let data = layer.data();
        match i {
            0 => {
                assert_eq!(data.name, "Background");
                assert_eq!(data.parallax_x, 0.5);
                assert_eq!(data.parallax_y, 0.75);
            }
            1 => {
                assert_eq!(data.name, "Middle");
                assert_eq!(data.parallax_x, 1.0);
                assert_eq!(data.parallax_y, 1.0);
            }
            2 => {
                assert_eq!(data.name, "Foreground");
                assert_eq!(data.parallax_x, 2.0);
                assert_eq!(data.parallax_y, 2.0);
            }
            _ => panic!("unexpected layer"),
        }
    }
}

#[test]
fn test_object_property() {
    let mut cache = DefaultResourceCache::new();

    let r = Map::parse_file("assets/tiled_object_property.tmx", &mut cache).unwrap();
    let layer = r.get_layer(1).unwrap();
    let prop_value = if let Some(PropertyValue::ObjectValue(v)) =
        as_object_layer(layer).data().objects[0]
            .properties
            .get("object property")
    {
        *v
    } else {
        0
    };
    assert_eq!(3, prop_value);
}
