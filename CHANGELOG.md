# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Reorganized crate:
    - `parse_file`, `parse` -> `Map::parse_file` with optional path.
    - `parse_with_path` -> `Map::parse_reader`
    - `parse_tileset` -> `Tileset::parse`
    - `Frame` has been moved to the `animation` module.
    - `ParseTileError` & `TiledError` have been moved into the `error` module.
    - `Image` has been moved into the `image` module.
    - `LayerTile`, `Layer`, `LayerData`, `ImageLayer` & `Chunk` have been moved into the `layers` module.
    - `Map` & `Orientation` have been moved into the `map` module.
    - `ObjectGroup`, `ObjectShape` & `Object` have been moved into the `objects` module.
    - `Colour`, `PropertyValue` & `Properties` have been moved into the `properties` module.
    - All mentions of `Colour` have been changed to `Color` for consistency with the Tiled dataformat.
    - `Tile` has been moved into the `tile` module.
    - `Tileset` has been moved into the `tileset` module.
    - `Map::get_tileset_by_gid` -> `Map::tileset_by_gid`
- Tile now has `image` instead of `images`. ([Issue comment](https://github.com/mapeditor/rs-tiled/issues/103#issuecomment-940773123))
- Tileset now has `image` instead of `images`.
- Functions that took in `&Path` now take `impl AsRef<Path>`.
- Bumped `zstd` to `0.9`.
- Fix markdown formatting in the `CONTRIBUTORS` file.

### Added
- `Map::source` for obtaining where the map actually came from.
- `Tileset::columns`.
- `layers::Layer::id`.
- Documentation for map members.
- Tests for `tiled_base64_zstandard.tmx`.
- `.gitattributes` for line ending consistency.
- Support for multiline string properties.
- MIT license file.


## [0.9.5]
TODO

## [0.9.4]
TODO

## [0.9.3]
TODO

## [0.9.2] - 2020-Apr-25
### Added
- Properties to Tilesets.
- Test verifying `tileset.properties`