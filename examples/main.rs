use std::path::PathBuf;

use tiled::Loader;

fn main() {
    let mut loader = Loader::new();

    let map_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("assets/tiled_base64_zlib.tmx");
    let map = loader.load_tmx_map(map_path).unwrap();

    for layer in map.layers() {
        print!("Layer \"{}\":\n\t", layer.name);

        match layer.layer_type() {
            tiled::LayerType::TileLayer(layer) => match layer {
                tiled::TileLayer::Finite(data) => println!(
                    "Finite tile layer with width = {} and height = {}; ID of tile @ (0,0): {}",
                    data.width(),
                    data.height(),
                    data.get_tile(0, 0).unwrap().id()
                ),
                tiled::TileLayer::Infinite(data) => {
                    println!(
                        "Infinite tile layer; Tile @ (-5, 0) = {:?}",
                        data.get_tile(-5, 0)
                    )
                }
            },
            tiled::LayerType::ObjectLayer(layer) => {
                println!("Object layer with {} objects", layer.objects().len())
            }
            tiled::LayerType::ImageLayer(layer) => {
                println!(
                    "Image layer with {}",
                    match &layer.image {
                        Some(img) =>
                            format!("an image with source = {}", img.source.to_string_lossy()),
                        None => "no image".to_owned(),
                    }
                )
            }
            tiled::LayerType::GroupLayer(layer) => {
                println!("Group layer with {} sublayers", layer.layers().len())
            }
        }
    }
}
