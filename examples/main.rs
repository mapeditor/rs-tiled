use std::path::PathBuf;

use tiled::{FilesystemResourceCache, Map};

fn main() {
    // Create a new resource cache. This is a structure that holds references to loaded
    // assets such as tilesets so that they only get loaded once.
    // [`FilesystemResourceCache`] is a implementation of [`tiled::ResourceCache`] that
    // identifies resources by their path in the filesystem.
    let mut cache = FilesystemResourceCache::new();

    let map_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("assets/tiled_base64_zlib.tmx");
    let map = Map::parse_file(map_path, &mut cache).unwrap();

    for layer in map.layers() {
        print!("Layer \"{}\":\n\t", layer.data().name);

        match layer.layer_type() {
            tiled::LayerType::TileLayer(layer) => match layer.data() {
                tiled::TileLayerData::Finite(data) => println!(
                    "Finite tile layer with width = {} and height = {}; ID of tile @ (0,0): {}",
                    data.width(),
                    data.height(),
                    layer.get_tile(0, 0).unwrap().id()
                ),
                tiled::TileLayerData::Infinite(data) => {
                    // This is prone to change! Infinite layers will be refactored before 0.10.0
                    // releases.
                    println!("Infinite tile layer with {} chunks", data.chunks.len())
                }
            },

            tiled::LayerType::ObjectLayer(layer) => {
                println!("Object layer with {} objects", layer.data().objects.len())
            }

            tiled::LayerType::ImageLayer(layer) => {
                println!(
                    "Image layer with {}",
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
