use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;
use xml::EventReader;

use crate::error::TiledError;
use crate::image::Image;
use crate::properties::{parse_properties, Properties};
use crate::tile::Tile;
use crate::util::*;

/// A tileset, usually the tilesheet image.
#[derive(Debug, PartialEq, Clone)]
pub struct Tileset {
    /// The GID of the first tile stored.
    pub(crate) first_gid: u32,
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
    pub tilecount: Option<u32>,
    pub columns: u32,

    /// A tileset can either:
    /// * have a single spritesheet `image` in `tileset` ("regular" tileset);
    /// * have zero images in `tileset` and one `image` per `tile` ("image collection" tileset).
    ///
    /// --------
    /// - Source: [tiled issue #2117](https://github.com/mapeditor/tiled/issues/2117)
    /// - Source: [`columns` documentation](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tileset)
    pub image: Option<Image>,
    special_tiles: HashMap<u32, Tile>,
    pub properties: Properties,

    /// Where this tileset was loaded from.
    /// If fully embedded (loaded with path = `None`), this will return `None`.
    pub source: Option<PathBuf>,
}

/// Internal structure for holding mid-parse information.
struct TilesetProperties {
    spacing: Option<u32>,
    margin: Option<u32>,
    tilecount: Option<u32>,
    columns: Option<u32>,
    first_gid: u32,
    name: String,
    tile_width: u32,
    tile_height: u32,
    path_relative_to: Option<PathBuf>,
    source: Option<PathBuf>,
}

impl Tileset {
    /// Parse a buffer hopefully containing the contents of a Tiled tileset.
    ///
    /// External tilesets do not have a firstgid attribute.  That lives in the
    /// map. You must pass in `first_gid`.  If you do not need to use gids for anything,
    /// passing in 1 will work fine.
    pub fn parse<R: Read>(reader: R, first_gid: u32) -> Result<Self, TiledError> {
        Tileset::new_external(reader, first_gid, None)
    }

    /// Parse a buffer hopefully containing the contents of a Tiled tileset.
    ///
    /// External tilesets do not have a firstgid attribute.  That lives in the
    /// map. You must pass in `first_gid`.  If you do not need to use gids for anything,
    /// passing in 1 will work fine.
    pub fn parse_with_path<R: Read>(
        reader: R,
        first_gid: u32,
        path: impl AsRef<Path>,
    ) -> Result<Self, TiledError> {
        Tileset::new_external(reader, first_gid, Some(path.as_ref()))
    }

    /// Gets a clone of the tile with the local ID specified, if it exists within this tileset.
    pub fn get_tile<'s: 't, 't>(&'s self, id: u32) -> Option<Tile> {
        if let Some(tile) = self.special_tiles.get(&id) {
            Some(tile.clone())
        } else {
            Some(Tile {
                id,
                ..Default::default()
            })
        }
    }
}

impl Tileset {
    pub(crate) fn parse_xml<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        path_relative_to: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        Tileset::parse_xml_embedded(parser, &attrs, path_relative_to).or_else(|err| {
            if matches!(err, TiledError::MalformedAttributes(_)) {
                Tileset::parse_xml_reference(&attrs, path_relative_to)
            } else {
                Err(err)
            }
        })
    }

    pub(crate) fn new_external<R: Read>(
        file: R,
        first_gid: u32,
        path: Option<&Path>,
    ) -> Result<Self, TiledError> {
        let mut tileset_parser = EventReader::new(file);
        loop {
            match tileset_parser
                .next()
                .map_err(TiledError::XmlDecodingError)?
            {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    if name.local_name == "tileset" {
                        return Self::parse_external_tileset(
                            first_gid,
                            &mut tileset_parser,
                            &attributes,
                            path,
                        );
                    }
                }
                XmlEvent::EndDocument => {
                    return Err(TiledError::PrematureEnd(
                        "Tileset Document ended before map was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }

    fn parse_xml_embedded<R: Read>(
        parser: &mut EventReader<R>,
        attrs: &Vec<OwnedAttribute>,
        path_relative_to: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        let ((spacing, margin, tilecount, columns), (first_gid, name, tile_width, tile_height)) = get_attrs!(
           attrs,
           optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
                ("tilecount", tilecount, |v:String| v.parse().ok()),
                ("columns", columns, |v:String| v.parse().ok()),
            ],
           required: [
            ("firstgid", first_gid, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        Self::finish_parsing_xml(
            parser,
            TilesetProperties {
                spacing,
                margin,
                name,
                path_relative_to: path_relative_to.map(Path::to_owned),
                columns,
                tilecount,
                tile_height,
                tile_width,
                first_gid,
                source: None,
            },
        )
    }

    fn parse_xml_reference(
        attrs: &Vec<OwnedAttribute>,
        path_relative_to: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        let ((), (first_gid, source)) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("firstgid", first_gid, |v:String| v.parse().ok()),
                ("source", name, |v| Some(v)),
            ],
            TiledError::MalformedAttributes("Tileset reference must have a firstgid and source with correct types".to_string())
        );

        let tileset_path = path_relative_to
            .ok_or(TiledError::SourceRequired {
                object_to_parse: "Tileset".to_string(),
            })?
            .join(source);
        let file = File::open(&tileset_path).map_err(|_| {
            TiledError::Other(format!(
                "External tileset file not found: {:?}",
                tileset_path
            ))
        })?;
        Tileset::new_external(file, first_gid, Some(&tileset_path))
    }

    fn parse_external_tileset<R: Read>(
        first_gid: u32,
        parser: &mut EventReader<R>,
        attrs: &Vec<OwnedAttribute>,
        path: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        let ((spacing, margin, tilecount, columns), (name, tile_width, tile_height)) = get_attrs!(
            attrs,
            optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
                ("tilecount", tilecount, |v:String| v.parse().ok()),
                ("columns", columns, |v:String| v.parse().ok()),
            ],
            required: [
                ("name", name, |v| Some(v)),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let source_path = path.and_then(|p| p.parent().map(Path::to_owned));

        Self::finish_parsing_xml(
            parser,
            TilesetProperties {
                spacing,
                margin,
                name,
                path_relative_to: source_path,
                columns,
                tilecount,
                tile_height,
                tile_width,
                first_gid,
                source: path.map(Path::to_owned),
            },
        )
    }

    fn finish_parsing_xml<R: Read>(
        parser: &mut EventReader<R>,
        prop: TilesetProperties,
    ) -> Result<Self, TiledError> {
        let mut image = Option::None;
        let mut tiles = HashMap::new();
        let mut properties = HashMap::new();
        parse_tag!(parser, "tileset", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, prop.path_relative_to.as_ref().ok_or(TiledError::SourceRequired{object_to_parse: "Image".to_string()})?)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "tile" => |attrs| {
                let tile = Tile::new(parser, attrs, prop.path_relative_to.as_ref().and_then(|p| Some(p.as_path())))?;
                tiles.insert(tile.id, tile);
                Ok(())
            },
        });

        let (margin, spacing) = (prop.margin.unwrap_or(0), prop.spacing.unwrap_or(0));

        let columns = prop
            .columns
            .map(Ok)
            .unwrap_or_else(|| Self::calculate_columns(&image, prop.tile_width, margin, spacing))?;

        Ok(Tileset {
            first_gid: prop.first_gid,
            name: prop.name,
            tile_width: prop.tile_width,
            tile_height: prop.tile_height,
            spacing,
            margin,
            columns,
            tilecount: prop.tilecount,
            image,
            special_tiles: tiles,
            properties,
            source: prop.source,
        })
    }

    fn calculate_columns(
        image: &Option<Image>,
        tile_width: u32,
        margin: u32,
        spacing: u32,
    ) -> Result<u32, TiledError> {
        image
            .as_ref()
            .ok_or(TiledError::MalformedAttributes(
                "No <image> nor columns attribute in <tileset>".to_string(),
            ))
            .and_then(|image| Ok((image.width as u32 - margin + spacing) / (tile_width + spacing)))
    }
}
