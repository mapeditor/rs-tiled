use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    parse::{
        common::tileset::{EmbeddedParseResult, EmbeddedParseResultType},
        xml::properties::parse_properties,
    },
    util::{get_attrs, parse_tag, XmlEventResult},
    Error, Gid, Image, ResourceCache, ResourceReader, Result, TileData, Tileset, WangSet,
};

/// Internal structure for holding mid-parse information.
struct TilesetProperties {
    spacing: Option<u32>,
    margin: Option<u32>,
    tilecount: u32,
    columns: Option<u32>,
    name: String,
    tile_width: u32,
    tile_height: u32,
    /// The root all non-absolute paths contained within the tileset are relative to.
    root_path: PathBuf,
}

impl Tileset {
    pub(crate) fn parse_xml(
        path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Tileset> {
        let mut tileset_parser = EventReader::new(reader.read_from(path).map_err(|err| {
            Error::ResourceLoadingError {
                path: path.to_owned(),
                err: Box::new(err),
            }
        })?);
        loop {
            match tileset_parser.next().map_err(Error::XmlDecodingError)? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == "tileset" => {
                    return Self::parse_external_tileset(
                        &mut tileset_parser.into_iter(),
                        &attributes,
                        path,
                        reader,
                        cache,
                    );
                }
                XmlEvent::EndDocument => {
                    return Err(Error::PrematureEnd(
                        "Tileset Document ended before map was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }

    pub(crate) fn parse_external_tileset(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: &[OwnedAttribute],
        path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Tileset> {
        let ((spacing, margin, columns, name), (tilecount, tile_width, tile_height)) = get_attrs!(
            for v in attrs {
                Some("spacing") => spacing ?= v.parse(),
                Some("margin") => margin ?= v.parse(),
                Some("columns") => columns ?= v.parse(),
                Some("name") => name = v,

                "tilecount" => tilecount ?= v.parse::<u32>(),
                "tilewidth" => tile_width ?= v.parse::<u32>(),
                "tileheight" => tile_height ?= v.parse::<u32>(),
            }
            ((spacing, margin, columns, name), (tilecount, tile_width, tile_height))
        );

        let root_path = path.parent().ok_or(Error::PathIsNotFile)?.to_owned();

        Self::finish_parsing_xml(
            parser,
            TilesetProperties {
                spacing,
                margin,
                name: name.unwrap_or_default(),
                root_path,
                columns,
                tilecount,
                tile_height,
                tile_width,
            },
            reader,
            cache,
        )
    }

    pub(crate) fn parse_xml_in_map(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: &[OwnedAttribute],
        path: &Path, // Template or Map file
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<EmbeddedParseResult> {
        Self::parse_xml_embedded(parser, attrs, path, reader, cache).or_else(|err| {
            if matches!(err, Error::MalformedAttributes(_)) {
                Self::parse_xml_reference(attrs, path)
            } else {
                Err(err)
            }
        })
    }

    pub(crate) fn parse_xml_embedded(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: &[OwnedAttribute],
        path: &Path, // Template or Map file
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<EmbeddedParseResult> {
        let ((spacing, margin, columns, name), (tilecount, first_gid, tile_width, tile_height)) = get_attrs!(
           for v in attrs {
            Some("spacing") => spacing ?= v.parse(),
            Some("margin") => margin ?= v.parse(),
            Some("columns") => columns ?= v.parse(),
            Some("name") => name = v,
            "tilecount" => tilecount ?= v.parse::<u32>(),
            "firstgid" => first_gid ?= v.parse::<u32>().map(Gid),
            "tilewidth" => tile_width ?= v.parse::<u32>(),
            "tileheight" => tile_height ?= v.parse::<u32>(),
           }
           ((spacing, margin, columns, name), (tilecount, first_gid, tile_width, tile_height))
        );

        let root_path = path.parent().ok_or(Error::PathIsNotFile)?.to_owned();

        Self::finish_parsing_xml(
            parser,
            TilesetProperties {
                spacing,
                margin,
                name: name.unwrap_or_default(),
                root_path,
                columns,
                tilecount,
                tile_height,
                tile_width,
            },
            reader,
            cache,
        )
        .map(|tileset| EmbeddedParseResult {
            first_gid,
            result_type: EmbeddedParseResultType::Embedded { tileset },
        })
    }

    fn parse_xml_reference(
        attrs: &[OwnedAttribute],
        map_path: &Path,
    ) -> Result<EmbeddedParseResult> {
        let (first_gid, source) = get_attrs!(
            for v in attrs {
                "firstgid" => first_gid ?= v.parse::<u32>().map(Gid),
                "source" => source = v,
            }
            (first_gid, source)
        );

        let tileset_path = map_path.parent().ok_or(Error::PathIsNotFile)?.join(source);

        Ok(EmbeddedParseResult {
            first_gid,
            result_type: EmbeddedParseResultType::ExternalReference { tileset_path },
        })
    }

    fn finish_parsing_xml(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        prop: TilesetProperties,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Tileset> {
        let mut image = Option::None;
        let mut tiles = HashMap::with_capacity(prop.tilecount as usize);
        let mut properties = HashMap::new();
        let mut wang_sets = Vec::new();

        parse_tag!(parser, "tileset", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, &prop.root_path)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "tile" => |attrs| {
                let (id, tile) = TileData::new(parser, attrs, &prop.root_path, reader, cache)?;
                tiles.insert(id, tile);
                Ok(())
            },
            "wangset" => |attrs| {
                let set = WangSet::new(parser, attrs)?;
                wang_sets.push(set);
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
        let columns = prop.columns.map(Ok).unwrap_or_else(|| {
            Tileset::calculate_columns(&image, prop.tile_width, margin, spacing)
        })?;

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
            wang_sets,
            properties,
        })
    }
}
