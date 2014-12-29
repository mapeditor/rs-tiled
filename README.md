# rs-tiled

![Travis](https://travis-ci.org/mattyhall/rs-tiled.svg?branch=master)

Read maps from the [Tiled Map Editor](http://www.mapeditor.org/) into rust for use in video games. It is game engine agnostic and pretty barebones at the moment. Documentation is available [on rust-ci](http://rust-ci.org/mattyhall/rs-tiled/doc/tiled/).

Code contributions are welcome as are bug reports, documentation, suggestions and criticism.


### How to use with Cargo
Whilst Rust < 1.0 the master branch of this repo will track rust nightly as closely as possible. This means that this library has to build its dependencies from git, as often the version of rust-xml on [crates.io](http://crates.io) does not compile with the latest version of rustc.

The long and the short of it being the recommended way of using rs-tiled is adding this to your ``Cargo.toml``:

```
[dependencies.tiled]
git = "https://github.com/mattyhall/rs-tiled.git"
```

[There is a package on crates.io](https://crates.io/crates/tiled) but it is unlikely to work until the language stabalises.


### Example

```rust
extern crate serialize;
extern crate tiled;

use std::io::File;
use std::io::BufferedReader;
use tiled::parse;

fn main() {
    let file = File::open(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    println!("Opened file");
    let reader = BufferedReader::new(file);
    let map = parse(reader).unwrap();
    println!("{}", map);
    println!("{}", map.get_tileset_by_gid(22));
}
```

### Things missing
There are a few things missing at the moment:

  * Terrain
  * Loading files that aren't base64 encoded and compressed with zlib
  * Tile flipping
  * Image layers
  * A nice API. At the moment you can access attributes and properties, find tilesets by GID and loop through the tiles. This leaves a user of the library with a bit to do.

### Licences
assets/tilesheet.png by Buch (http://blog-buch.rhcloud.com/)

Licenced under MIT
