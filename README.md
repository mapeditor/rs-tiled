# rs-tiled
```toml
tiled = "0.11.0"
```

[![Rust](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml/badge.svg)](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/tiled.svg)](https://crates.io/crates/tiled)
[![dependency status](https://deps.rs/crate/tiled/latest/status.svg)](https://deps.rs/crate/tiled)

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

### WASM
The crate supports WASM, but since it does not currently support asynchronous loading, there are some gotchas.

- First, to make it work on any WASM target, **enable the wasm feature**, like so:
```toml
[dependencies]
# ...
tiled = { version = ".....", features = ["wasm"] }
```

- Second, since you cannot use the filesystem as normally on the web, you cannot use `FilesystemResourceReader`. As such,
you'll need to implement your own `ResourceReader`. This is a pretty simple task, as you just need to return anything
that is `Read`able when given a path, e.g.:
```rust
use std::io::Cursor;

struct MyReader;

impl tiled::ResourceReader for MyReader {
    type Resource = Cursor<&'static [u8]>;
    type Error = std::io::Error;

    // really dumb example implementation that just keeps resources in memory
    fn read_from(&mut self, path: &std::path::Path) -> std::result::Result<Self::Resource, Self::Error> {
        if path == std::path::Path::new("my_map.tmx") {
            Ok(Cursor::new(include_bytes!("../assets/tiled_xml.tmx")))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
        }
    }
}
```
Check the `ResourceReader` docs for more information.

### Licences

assets/tilesheet.png by [Buch](https://opengameart.org/content/sci-fi-interior-tiles)

Licenced under MIT
