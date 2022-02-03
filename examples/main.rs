use std::path::PathBuf;

use tiled::{DefaultResourceCache, Map};

fn main() {
    let mut tilesets = DefaultResourceCache::new();

    let map = Map::parse_file(
        PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("assets/tiled_base64_zlib.tmx"),
        &mut tilesets,
    )
    .unwrap();

    for layer in map.layers() {
        println!("Layer \"{}\":", layer.data().name);
        match layer.layer_type() {
            tiled::LayerType::TileLayer(layer) => match layer.data() {
                tiled::TileLayerData::Finite(data) => println!(
                    "\tFinite tile layer with width = {} and height = {}",
                    data.width(),
                    data.height()
                ),
                tiled::TileLayerData::Infinite(data) => {
                    println!("\tInfinite tile layer with {} chunks", data.chunks.len())
                }
            },
            tiled::LayerType::ObjectLayer(layer) => {
                println!("\tObject layer with {} objects", layer.data().objects.len())
            }
            tiled::LayerType::ImageLayer(layer) => {
                println!(
                    "\tImage layer with {}",
                    match &layer.data().image {
                        Some(img) =>
                            format!("an image with source = {}", img.source.to_string_lossy()),
                        None => "no image".to_owned(),
                    }
                )
            }
        }
    }
}
