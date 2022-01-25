# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `Tileset::source` for obtaining where the tileset actually came from.
- `Tileset::columns`.
- `Layer::id`, `Layer::width` and `Layer::height`.
- Support for 'object'-type properties.
- Documentation for map members.
- Tests for `tiled_base64_zstandard.tmx`.
- `.gitattributes` for line ending consistency.
- Support for multiline string properties.
- MIT license file.

### Changed
- Reorganized crate:
    - `parse_file`, `parse` -> `Map::parse_file` with optional path.
    - `parse_with_path` -> `Map::parse_reader`
    - `parse_tileset` -> `Tileset::parse`
    - `Frame` has been moved to the `animation` module.
    - `ParseTileError` & `TiledError` have been moved into the `error` module.
    - `Image` has been moved into the `image` module.
    - `LayerTile`, `LayerData`, `ImageLayer` & `Chunk` have been moved into the `layers` module.
    - `Layer` has been renamed to `TileLayer` and has been moved into the `layers` module.
    - `ObjectGroup` has been renamed to `ObjectLayer` and now resides in the `layers` module.
    - `Map` & `Orientation` have been moved into the `map` module.
    - `ObjectShape` & `Object` have been moved into the `objects` module.
    - `Colour`, `PropertyValue` & `Properties` have been moved into the `properties` module.
    - All mentions of `Colour` have been changed to `Color` for consistency with the Tiled dataformat.
    - `Tile` has been moved into the `tile` module.
    - `Tileset` has been moved into the `tileset` module.
    - `Map::get_tileset_by_gid` -> `Map::tileset_by_gid`
- `Map` now has a single `layers` member which contains layers of all types in order.
- Layer members that are common between types (i.e. `id`, `name`, `visible`, `opacity`, `offset_x`,
  `offset_y` and `properties`) have been moved into `Layer`.
- `Layer::tiles` changed from `Vec<Vec<LayerTile>>` to `Vec<LayerTile>`.
- Tile now has `image` instead of `images`. ([Issue comment](https://github.com/mapeditor/rs-tiled/issues/103#issuecomment-940773123))
- Tileset now has `image` instead of `images`.
- `Image::source` is now a `PathBuf` instead of a `String`.
- Functions that took in `&Path` now take `impl AsRef<Path>`.
- Refactored internals.
- Fixed library warnings.
- Bumped `zstd` to `0.9`.
- Fixed markdown formatting in the `CONTRIBUTORS` file.

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
