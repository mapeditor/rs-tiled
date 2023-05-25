# rs-tiled
```toml
tiled = "0.11.1"
```

[![Rust](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml/badge.svg)](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/tiled.svg)](https://crates.io/crates/tiled)
[![Docs Status](https://docs.rs/tiled/badge.svg)](https://docs.rs/tiled)
[![dependency status](https://deps.rs/crate/tiled/latest/status.svg)](https://deps.rs/crate/tiled)

A crate for reading TMX (map) and TSX (tileset) files from the [Tiled Map Editor](http://www.mapeditor.org/) into Rust.
It provides a huge set of features as well as a strong wrapper over internal features such as GIDs.

Documentation is available [on docs.rs](https://docs.rs/tiled/).

Code contributions are welcome as are bug reports, documentation, suggestions and criticism.

The minimum supported TMX version is 0.13.

## Example

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

## FAQ
### How do I embed a map into my executable? / How do I read a file from anywhere else that isn't the filesystem's OS?
The crate does all of its reading through the `read_from` function of the [`ResourceReader`](https://docs.rs/tiled/latest/tiled/trait.ResourceReader.html) that you create the loader with. By default, this reader is set to [`FilesystemResourceReader`](https://docs.rs/tiled/latest/tiled/struct.FilesystemResourceReader.html) and all files are read through the OS's filesystem. You can however change this.

Here's an example mostly taken from `Loader::with_cache_and_reader`'s documentation:
```rust
use tiled::{DefaultResourceCache, Loader};

let mut loader = Loader::with_cache_and_reader(
    DefaultResourceCache::new(),
    // Specify the reader to use. We can use anything that implements `ResourceReader`, e.g. FilesystemResourceReader.
    // Any function that has the same signature as `ResourceReader::read_from` also implements it.
    // Here we define a reader that embeds the map at "assets/tiled_xml.csv" into the executable, and allow
    // accessing it only through "/my-map.tmx"
    // ALL maps, tilesets and templates will be read through this function, even if you don't explicitly load them
    // (They can be dependencies of one you did want to load in the first place).
    // Doing this embedding is useful for places where the OS filesystem is not available (e.g. WASM applications).
    |path: &std::path::Path| -> std::io::Result<_> {
        if path == std::path::Path::new("/my-map.tmx") {
            Ok(std::io::Cursor::new(include_bytes!("../assets/tiled_csv.tmx")))
        } else {
            Err(std::io::ErrorKind::NotFound.into())
        }
    }
);
```
If the closure approach confuses you or you need more flexibility, you can always implement [`ResourceReader`](https://docs.rs/tiled/latest/tiled/trait.ResourceReader.html) on your own structure.

### How do I get the crate to work on WASM targets?
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
