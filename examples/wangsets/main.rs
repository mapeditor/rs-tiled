use std::{env, path::PathBuf};
use tiled::Loader;

const MAP_PATH: &str = "assets/Chapter1.tmx";

fn main() {
    let mut loader = Loader::new();
        let map = loader
        .load_tmx_map(
            PathBuf::from(
                env::var("CARGO_MANIFEST_DIR")
                    .expect("To run the example, use `cargo run --example wangsets`"),
            )
            .join(MAP_PATH),
        )
        .unwrap();
    for w in map.tilesets().get(0).unwrap().wangsets.iter() {
        println!("{:?}", w);
    }
}
