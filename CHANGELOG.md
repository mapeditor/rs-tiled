# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `Tileset::source` for obtaining where the tileset actually came from.
- `Tileset::columns`.
- `Color::alpha`.
- `Layer::id`, `Layer::width`, `Layer::height`, `Layer::parallax_x` and `Layer::parallax_y`.
- Support for 'object'-type properties.
- Support for multiline string properties.
- Documentation for map members.
- Tests for `tiled_base64_zstandard.tmx`.
- `.gitattributes` for line ending consistency.
- MIT license file.

### Changed
- **Set the minimum Tiled TMX version to 0.13.**
- `Tileset::tilecount` is no longer optional.
- `Layer` has been renamed to `TileLayer`, and the original `Layer` structure is now used
  for common data from all layer types.
- `Map` now has a single `layers` member which contains layers of all types in order.
- Layer members that are common between types (i.e. `id`, `name`, `visible`, `opacity`, `offset_x`,
  `offset_y` and `properties`) have been moved into `Layer`.
- `ObjectGroup` has been renamed to `ObjectLayer`.
- `parse_file`, `parse` -> `Map::parse_file` with optional path.
- `parse_with_path` -> `Map::parse_reader`.
- `parse_tileset` -> `Tileset::parse`.
- All mentions of `Colour` have been changed to `Color` for consistency with the Tiled dataformat.
- `Layer::tiles` changed from `Vec<Vec<LayerTile>>` to `Vec<LayerTile>`.
- Tile now has `image` instead of `images`. ([Issue comment](https://github.com/mapeditor/rs-tiled/issues/103#issuecomment-940773123))
- Tileset now has `image` instead of `images`.
- `Image::source` is now a `PathBuf` instead of a `String`.
- Functions that took in `&Path` now take `impl AsRef<Path>`.
- Refactored internals.
- Fixed library warnings.
- Bumped `zstd` to `0.9`.
- Fixed markdown formatting in the `CONTRIBUTORS` file.

### Fixed
- `Color` parsing.


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
