# rs-tiled
```toml
tiled = "0.10.2"
```

[![Rust](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml/badge.svg)](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/tiled.svg)](https://crates.io/crates/tiled)
[![dependency status](https://deps.rs/crate/tiled/0.10.1/status.svg)](https://deps.rs/crate/tiled/0.10.1)

A crate for reading TMX (map) and TSX (tileset) files from the [Tiled Map Editor](http://www.mapeditor.org/) into Rust.
It provides a huge set of features as well as a strong wrapper over internal features such as GIDs.

Documentation is available [on docs.rs](https://docs.rs/tiled/).

Code contributions are welcome as are bug reports, documentation, suggestions and criticism.

The minimum supported TMX version is 0.13.

### Example

```rust
use tiled::Loader;

fn main() {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("assets/tiled_base64_zlib.tmx").unwrap();
    println!("{:?}", map);
    println!("{:?}", map.tilesets()[0].get_tile(0).unwrap().probability);
    
    let tileset = loader.load_tsx_tileset("assets/tilesheet.tsx").unwrap();
    assert_eq!(*map.tilesets()[0], tileset);
}

```

### Licences

assets/tilesheet.png by [Buch](https://opengameart.org/content/sci-fi-interior-tiles)

Licenced under MIT
