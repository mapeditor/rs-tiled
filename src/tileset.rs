use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;
use xml::EventReader;

use crate::error::TiledError;
use crate::image::Image;
use crate::properties::{parse_properties, Properties};
use crate::tile::Tile;
use crate::{util::*, Gid};

/// A tileset, usually the tilesheet image.
#[derive(Debug, PartialEq, Clone)]
pub struct Tileset {
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
    pub tilecount: u32,
    pub columns: u32,

    /// A tileset can either:
    /// * have a single spritesheet `image` in `tileset` ("regular" tileset);
    /// * have zero images in `tileset` and one `image` per `tile` ("image collection" tileset).
    ///
    /// --------
    /// - Source: [tiled issue #2117](https://github.com/mapeditor/tiled/issues/2117)
    /// - Source: [`columns` documentation](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tileset)
    pub image: Option<Image>,

    /// All the tiles present in this tileset, indexed by their local IDs.
    pub tiles: HashMap<u32, Tile>,

    /// The custom properties of the tileset.
    pub properties: Properties,
}

pub(crate) enum EmbeddedParseResultType {
    ExternalReference { tileset_path: PathBuf },
    Embedded { tileset: Tileset },
}

pub(crate) struct EmbeddedParseResult {
    pub first_gid: Gid,
    pub result_type: EmbeddedParseResultType,
}

/// Internal structure for holding mid-parse information.
struct TilesetProperties {
    spacing: Option<u32>,
    margin: Option<u32>,
    tilecount: u32,
    columns: Option<u32>,
    name: String,
    tile_width: u32,
    tile_height: u32,
    /// The path all non-absolute paths are relative to.
    path_relative_to: PathBuf,
}

impl Tileset {
    /// Parse a buffer hopefully containing the contents of a Tiled tileset.
    pub fn parse_with_path<R: Read>(reader: R, path: impl AsRef<Path>) -> Result<Self, TiledError> {
        Tileset::new_external(reader, path.as_ref())
    }
}

impl Tileset {
    pub fn get_tile(&self, id: u32) -> Option<&Tile> {
        self.tiles.get(&id)
    }

    pub(crate) fn new_external<R: Read>(file: R, path: &Path) -> Result<Self, TiledError> {
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
                            &mut tileset_parser.into_iter(),
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

    pub(crate) fn parse_xml_in_map(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        map_path: &Path,
    ) -> Result<EmbeddedParseResult, TiledError> {
        Tileset::parse_xml_embedded(parser, &attrs, map_path).or_else(|err| {
            if matches!(err, TiledError::MalformedAttributes(_)) {
                Tileset::parse_xml_reference(&attrs, map_path)
            } else {
                Err(err)
            }
        })
    }

    /// Returns both the tileset and its first gid in the corresponding map.
    fn parse_xml_embedded(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: &Vec<OwnedAttribute>,
        map_path: &Path,
    ) -> Result<EmbeddedParseResult, TiledError> {
        let ((spacing, margin, columns), (tilecount, first_gid, name, tile_width, tile_height)) = get_attrs!(
           attrs,
           optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
                ("columns", columns, |v:String| v.parse().ok()),
            ],
           required: [
                ("tilecount", tilecount, |v:String| v.parse().ok()),
                ("firstgid", first_gid, |v:String| v.parse().ok().map(|n| Gid(n))),
                ("name", name, |v| Some(v)),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let dir_containing_map = map_path.parent().ok_or(TiledError::PathIsNotFile)?;

        Self::finish_parsing_xml(
            parser,
            TilesetProperties {
                spacing,
                margin,
                name,
                path_relative_to: dir_containing_map.to_owned(),
                columns,
                tilecount,
                tile_height,
                tile_width,
            },
        )
        .map(|tileset| EmbeddedParseResult {
            first_gid,
            result_type: EmbeddedParseResultType::Embedded { tileset },
        })
    }

    /// Returns both the tileset and its first gid in the corresponding map.
    fn parse_xml_reference(
        attrs: &Vec<OwnedAttribute>,
        map_path: &Path,
    ) -> Result<EmbeddedParseResult, TiledError> {
        let ((), (first_gid, source)) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("firstgid", first_gid, |v:String| v.parse().ok().map(|n| Gid(n))),
                ("source", name, |v| Some(v)),
            ],
            TiledError::MalformedAttributes("Tileset reference must have a firstgid and source with correct types".to_string())
        );

        let dir_containing_map = map_path.parent().ok_or(TiledError::PathIsNotFile)?;
        let tileset_path = dir_containing_map.join(source);

        Ok(EmbeddedParseResult {
            first_gid,
            result_type: EmbeddedParseResultType::ExternalReference { tileset_path },
        })
    }

    fn parse_external_tileset(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: &Vec<OwnedAttribute>,
        path: &Path,
    ) -> Result<Tileset, TiledError> {
        let ((spacing, margin, columns), (tilecount, name, tile_width, tile_height)) = get_attrs!(
            attrs,
            optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
                ("columns", columns, |v:String| v.parse().ok()),
            ],
            required: [
                ("tilecount", tilecount, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let path_relative_to = path
            .parent()
            .ok_or(TiledError::PathIsNotFile)
            .map(Path::to_owned)?;

        Self::finish_parsing_xml(
            parser,
            TilesetProperties {
                spacing,
                margin,
                name,
                path_relative_to,
                columns,
                tilecount,
                tile_height,
                tile_width,
            },
        )
    }

    fn finish_parsing_xml(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        prop: TilesetProperties,
    ) -> Result<Tileset, TiledError> {
        let mut image = Option::None;
        let mut tiles = HashMap::with_capacity(prop.tilecount as usize);
        let mut properties = HashMap::new();

        parse_tag!(parser, "tileset", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, &prop.path_relative_to)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "tile" => |attrs| {
                let (id, tile) = Tile::new(parser, attrs, &prop.path_relative_to)?;
                tiles.insert(id, tile);
                Ok(())
            },
        });

        // A tileset is considered an image collection tileset if there is no image attribute (because its tiles do).
        let is_image_collection_tileset = image.is_none();

        if !is_image_collection_tileset {
            for tile_id in 0..prop.tilecount {
                tiles.entry(tile_id).or_default();
            }
        }

        let margin = prop.margin.unwrap_or(0);
        let spacing = prop.spacing.unwrap_or(0);
        let columns = prop
            .columns
            .map(Ok)
            .unwrap_or_else(|| Self::calculate_columns(&image, prop.tile_width, margin, spacing))?;

        Ok(Tileset {
            name: prop.name,
            tile_width: prop.tile_width,
            tile_height: prop.tile_height,
            spacing,
            margin,
            columns,
            tilecount: prop.tilecount,
            image,
            tiles,
            properties,
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
