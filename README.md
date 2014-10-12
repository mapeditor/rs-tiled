# rs-tiled

![Travis](https://travis-ci.org/mattyhall/rs-tiled.svg?branch=master)

Read maps from the [Tiled Map Editor](http://www.mapeditor.org/) into rust for use in video games. It is game engine agnostic and pretty barebones at the moment. Documentation is available [on rust-ci](http://rust-ci.org/mattyhall/rs-tiled/doc/tiled/).

Code contributions are welcome as are bug reports, documentation, suggestions and critism.


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

  * Storing any colour - eg. transparency colours on images or background colours on maps.
  * Terrain
  * Loading files that aren't base64 encoded and compressed with zlib
  * Tile flipping
  * Image layers
  * A nice API. At the moment you can access attributes and properties, find tilesets by GID and loop through the tiles. This leaves a user of the library with a bit to do.

### Licences
assets/tilesheet.png by Buch (http://blog-buch.rhcloud.com/)

Licenced under MIT
