# rs-tiled
```toml
tiled = "0.10.0"
```

[![Rust](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml/badge.svg)](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/tiled.svg)](https://crates.io/crates/tiled)

A crate for reading TMX (map) and TSX (tileset) files from the [Tiled Map Editor](http://www.mapeditor.org/) into Rust.
It provides a huge set of features as well as a strong wrapper over internal features such as GIDs.

Documentation is available [on docs.rs](https://docs.rs/tiled/).

Code contributions are welcome as are bug reports, documentation, suggestions and criticism.

The minimum supported TMX version is 0.13.

### Example

```rust
use tiled::{FilesystemResourceCache, Map};

fn main() {
    let map = Map::parse_file(
        "assets/tiled_base64_zlib.tmx",
        &mut FilesystemResourceCache::new(),
    )
    .unwrap();
    println!("{:?}", map);
    println!("{:?}", map.tilesets()[0].get_tile(0).unwrap().probability);
}

```

### Licences

assets/tilesheet.png by [Buch](https://opengameart.org/content/sci-fi-interior-tiles)

Licenced under MIT
