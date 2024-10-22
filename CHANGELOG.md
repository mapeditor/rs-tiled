# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased (0.13.0)]
### Added
- Added a `source` member to `Tileset`, `Map` and `Template`, which stores the resource path they have been loaded from.

## [Unreleased (0.12.2)]
### Fixed
- Fixed template instance size and position overrides in `ObjectData::shape`. (#309)
- Add `hexside_length` members to `Map`. (#313)

## [0.12.1]
### Changed
- Improved documentation on `Map::layers` and `Map::get_layer`. (#306)
- Extend lifetime of `Layer`s returned from `GroupLayer::get_layer` to the map's. (#307)

## [0.12.0]
### Added
- Add `text`, `width` and `height` members to `ObjectShape::Text`. (#278)
- Implement `ResourceReader` for appropiate functions. (#272) **Read the README's FAQ for more information about this change.**
- Support for custom type properties. (#283)

### Changed
- Underlying reader for maps now uses a BufReader, which should give a large performance boost. (#286)
- Update `base64` to `0.22.1`. (#294)
- Update `libflate` to `2.1.0`. (#294)
- Update `zstd` to `0.13.1`. (#294)

### Fixed
- `ObjectShape::Text::kerning`'s default value, which should have been set to `true` instead of `false`. (#278)
- Unhandled u32 parsing panic in `decode_csv`. (#288)
- Panic in `<Color as FromStr>::from_str` when parsing non-ascii input. (#290)
- Index out of bounds in `InfiniteTileLayerData` when parsing a chunk. (#289)
- Panic on unexpected XML in `ObjectData` content. (#291)
- Divide by zero when parsing a tileset with height/width dimension of 0. (#292)

## [0.11.3]
### Changed
- Replace `libflate` with `flate2`. (#281)

## [0.11.2]
### Changed
- Updated `Image` docs. (#270)
- Update `libflate` dependency to `2.0.0`. (#279)
- Fix some doc links. (#273)
- Update ggez example to 0.9.3.

## [0.11.1]
### Added
- WASM support + `wasm` feature; Check `README.md` for instructions on how to set it up. (#252)
- Support for staggered maps. Maps now have an `stagger_axis` and `stagger_index` property. (#262)

## [0.11.0]
### Added
- Template support!
Templates are loaded automatically when they are encountered, and are treated as intermediate
objects. As such, `ResourceCache` has now methods for both getting and inserting them (#170).
- VFS support (#199).
- `as_x` functions for layer types (#235).
- Text object support (#230).
- `cache_mut` loader property (#207).

### Changed
- `LayerType` variants have been stripped from the `Layer` suffix (#203).
- `TileData::tile_type` has been renamed to `TileData::user_type`. (#253)
- `Orientation`'s `FromStr` impl now returns `OrientationParseError` as the error type. (#253)
- `ResourceCache::get_or_try_insert_tileset_with` has been replaced by `ResourceCache::insert_tileset`.
- `DefaultResourceCache`'s members have been made public.

### Removed
- `ObjectData::obj_type`, `ObjectData::width`, `ObjectData::height`. (#253)
- `TileData::tile_type` (which has been renamed to `TileData::user_type`) (#253)
- `Loader::load_tmx_map_from`, `Loader::load_tsx_tileset_from`: This was a hacky solution to loading embedded maps and tilesets. While there is no direct, ready-to-use replacement for this within the crate, you can create a `ResourceReader` that works like this function previously did:
```rs
struct EmbeddedResourceReader<R: Read> {
    reader: Option<R>,
}

impl<R: Read> EmbeddedResourceReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader: Some(reader),
        }
    }
}

impl<R: Read> tiled::ResourceReader for EmbeddedResourceReader<R> {
    type Resource = R;
    type Error = std::io::Error;

    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        if path == "the_path_to_load.tmx" {
            if let Some(x) = self.reader.take() {
                Ok(x)
            } else {
                Err(std::io::Error::from(ErrorKind::ResourceBusy))
            }
        } else {
            Err(std::io::Error::from(ErrorKind::NotFound))
        }
    }
}
```
Note: This works for both maps and tilesets, and you can extend to use more than one reader if neccessary, which is something you couldn't do before.

## [0.10.3]
### Added
- Support for Wang sets.
- Support for Tiled 1.9 `Class` property. Maps, tilesets and layers now have a `user_type` property.
- Support for tile offsets. Tilesets now have an `offset_x` and `offset_y` property.

### Deprecated
- `Object::obj_type` Use `Object::user_type` instead.

### Changed
- Update `zstd` to `0.12.0`.
- Update `sfml` dev dependency to `0.20.0`.
- Update `base64` to `0.21.0`.

## [0.10.2]
### Added
- Map-wrapped chunks: `ChunkWrapper`.
- Ability to access infinite tile layer chunks via `InfiniteTileLayer::chunks`, 
`InfiniteTileLayerData::chunk_data`, `InfiniteTileLayerData::get_chunk_data` &
`InfiniteTileLayer::get_chunk`, as well as chunk data via `Chunk::get_tile_data` &
`ChunkWrapper::get_tile`.
- `TileLayer::width` & `TileLayer::height` for ergonomic access of width/height.
- `FiniteTileLayerData::get_tile_data`, `InfiniteTileLayerData::get_tile_data`.
- `Default` derived implementation for `Loader` & `FilesystemResourceCache`

### Changed
- Update `zstd` to `0.11.0`.

## [0.10.1]
### Added
- `Loader` type for loading map and tileset files without having to necessarily mention the cache
to use.

### Deprecated
- `Map::parse_reader`: Use `Loader::parse_tmx_map_from` instead.
- `Map::parse_file`: Use `Loader::load_tmx_map` instead.
- `Tileset::parse_reader`: Use `Loader::load_tsx_tileset` instead.

### Fixed
- Fix message when a tileset is missing the `tilecount` attribute (#194).

## [0.10.0]
As this release changes practically the entire interface of the crate, it is recommended that you
check out the [examples](https://github.com/mapeditor/rs-tiled/tree/master/examples) instead of the
changelog if you are migrating from an older version.

### Added
- Documentation to all crate items.
- Group layer support.
- Layer ID parsing.
- Object property parsing.
- Support for multiline string properties.
- SFML example.
- `Result` type.
- `Layer::parallax_x` & `Layer::parallax_y`.
- `Tileset::columns`.
- Missing derive and inline attributes.
- Tests for `zstd`-compressed files.


### Changed
- **Set the minimum Tiled TMX version to 0.13.**
- Refactor crate interface and internals to be more consistent, sound and easy to use.
- Hide GIDs as internal data; Provide a cleaner API.
- Contain all layer types in an enum as opposed to different containers.
- Rename `TiledError` to `Error`.
- `Tileset::tilecount` is no longer optional.
- Improve errors.
- Use `Color` type in color properties.
- Rename "colour"-related appareances to "color".
- Use `impl AsRef<Path>` where appropiate.
- Change `Tileset::image` to be a single image at most instead of a vector.
- Update README.
- Make layer and tileset names optional, defaulting to an empty string.
- Reorganize crate internally.
- Update `zstd` to `0.9`.

### Fixed
- Color parsing issues: #148

### Removed
- `Layer::layer_index`, as all layer types are now stored in a common container.
- `Map::source`, since it is known from where the load function was called.

## [0.9.5] - 2021-05-02
### Added
- Support for file properties.

### Fixed
- Parsing csv data without newlines (LDtk).

## [0.9.4] - 2021-02-07
### Added
- Support for layer offsets.

### Changed
- Feature gate zstd to allow targeting wasm32-unknown-unknown.

### Fixed
- Object visibility parsing.

## [0.9.3] - 2020-09-20
### Added
- Support for base64 and zstd compressed maps.
- Support for point objects.
- Support for infinite maps.

## [0.9.2] - 2020-05-09
### Added
- Properties to Tilesets.
- Test verifying `tileset.properties`.
- Tileset tile count parsing.
- Object `width` and `height` fields.

## [0.9.1] - 2020-03-29
### Changed
- Make fields on `Frame` `pub`.

## [0.9.0] - 2019-25-11 (?)
### Changed
- Migration to `rust 2018` and `?`
