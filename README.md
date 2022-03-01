# rs-tiled

[![Rust](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml/badge.svg)](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/tiled.svg)](https://crates.io/crates/tiled)

Read maps from the [Tiled Map Editor](http://www.mapeditor.org/) into rust for use in video games. It is game engine agnostic and pretty barebones at the moment. Documentation is available [on docs.rs](https://docs.rs/tiled/).

Code contributions are welcome as are bug reports, documentation, suggestions and criticism.

[There is a package on crates.io](https://crates.io/crates/tiled), to use simply add:

```toml
tiled = "0.9.5"
```

to the dependencies section of your Cargo.toml.

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
    println!("{:?}", map.tilesets()[0].get_tile(0).unwrap().probability());
}

```

### Licences

assets/tilesheet.png by Buch (https://opengameart.org/content/sci-fi-interior-tiles)

Licenced under MIT
