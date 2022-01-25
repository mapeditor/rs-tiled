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
- Support for multiline string properties.
- Tests for `tiled_base64_zstandard.tmx`.

### Changed
- `parse_file`, `parse` -> `Map::parse_file` with optional path.
- `parse_with_path` -> `Map::parse_reader`.
- `parse_tileset` -> `Tileset::parse`.
- All mentions of `Colour` have been changed to `Color` for consistency with the Tiled dataformat.
- `Map::get_tileset_by_gid` -> `Map::tileset_by_gid`.
- `Layer::tiles` changed from `Vec<Vec<LayerTile>>` to `Vec<LayerTile>`.
- Tile now has `image` instead of `images`. ([Issue comment](https://github.com/mapeditor/rs-tiled/issues/103#issuecomment-940773123))
- Tileset now has `image` instead of `images`.
- `Image::source` is now a `PathBuf` instead of a `String`.
- Functions that took in `&Path` now take `impl AsRef<Path>`.
- Bumped `zstd` to `0.9`.

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
