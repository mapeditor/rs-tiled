# rs-tiled

[![Rust](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml/badge.svg)](https://github.com/mapeditor/rs-tiled/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/tiled.svg)](https://crates.io/crates/tiled)

Read maps from the [Tiled Map Editor](http://www.mapeditor.org/) into rust for use in video games. It is game engine agnostic and pretty barebones at the moment. Documentation is available [on docs.rs](https://docs.rs/tiled/).

Code contributions are welcome as are bug reports, documentation, suggestions and criticism.

[There is a package on crates.io](https://crates.io/crates/tiled), to use simply add:

```
tiled = "0.9.5"
```

to the dependencies section of your Cargo.toml.

### Example

```rust
use std::path::Path;
use tiled::parse_file;

fn main() {
    let map = parse_file(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    println!("Opened file");
    println!("{:?}", map);
    println!("{:?}", map.get_tileset_by_gid(22));
}
```

### Licences

assets/tilesheet.png by Buch (https://opengameart.org/content/sci-fi-interior-tiles)

Licenced under MIT
