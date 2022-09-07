use std::{collections::HashMap, path::Path};

use xml::attribute::OwnedAttribute;

use crate::{
    parse::xml::properties::parse_properties,
    parse_animation,
    util::{get_attrs, parse_tag, XmlEventResult},
    Image, ObjectLayerData, ResourceCache, ResourceReader, Result, TileData, TileId,
};

impl TileData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        path_relative_to: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<(TileId, TileData)> {
        let ((tile_type, probability), id) = get_attrs!(
            for v in attrs {
                Some("type") => tile_type ?= v.parse(),
                Some("probability") => probability ?= v.parse(),
                "id" => id ?= v.parse::<u32>(),
            }
            ((tile_type, probability), id)
        );

        let mut image = Option::None;
        let mut properties = HashMap::new();
        let mut objectgroup = None;
        let mut animation = None;
        parse_tag!(parser, "tile", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, path_relative_to)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "objectgroup" => |attrs| {
                // Tile objects are not allowed within tile object groups, so we can pass None as the
                // tilesets vector
                objectgroup = Some(ObjectLayerData::new(parser, attrs, None, None, path_relative_to, reader, cache)?.0);
                Ok(())
            },
            "animation" => |_| {
                animation = Some(parse_animation(parser)?);
                Ok(())
            },
        });
        Ok((
            id,
            TileData {
                image,
                properties,
                collision: objectgroup,
                animation,
                tile_type,
                probability: probability.unwrap_or(1.0),
            },
        ))
    }
}
