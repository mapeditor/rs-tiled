use std::path::PathBuf;

use glob::glob;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus_dir = path.join("corpus/tiled");
    std::fs::create_dir_all(&corpus_dir).expect("failed creating corpus dir");

    let tmx_assets_path = path.join("../assets/*.tmx");
    for entry in glob(tmx_assets_path.to_str().unwrap()).unwrap() {
        match entry {
            Ok(src) => {
                let dest = corpus_dir.join(src.file_name().unwrap());
                std::fs::copy(src, dest).unwrap();
            }
            Err(e) => {
                p!("{:?}", e)
            }
        }
    }
}
