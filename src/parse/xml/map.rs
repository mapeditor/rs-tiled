use std::{collections::HashMap, path::Path, sync::Arc};

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    parse::{common::tileset::EmbeddedParseResultType, xml::properties::parse_properties},
    util::{get_attrs, parse_tag, XmlEventResult},
    Error, LayerData, LayerTag, Map, MapTilesetGid, Orientation, ResourceCache, ResourceReader,
    Result, Tileset,
};

pub fn parse_map(
    path: &Path,
    reader: &mut impl ResourceReader,
    cache: &mut impl ResourceCache,
) -> Result<Map> {
    let mut parser =
        EventReader::new(
            reader
                .read_from(path)
                .map_err(|err| Error::ResourceLoadingError {
                    path: path.to_owned(),
                    err: Box::new(err),
                })?,
        );
    loop {
        match parser.next().map_err(Error::XmlDecodingError)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name == "map" {
                    return Map::parse_xml(
                        &mut parser.into_iter(),
                        attributes,
                        path,
                        reader,
                        cache,
                    );
                }
            }
            XmlEvent::EndDocument => {
                return Err(Error::PrematureEnd(
                    "Document ended before map was parsed".to_string(),
                ))
            }
            _ => {}
        }
    }
}

impl Map {
    pub(crate) fn parse_xml(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        map_path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Map> {
        let ((c, infinite), (v, o, w, h, tw, th)) = get_attrs!(
            for v in attrs {
                Some("backgroundcolor") => colour ?= v.parse(),
                Some("infinite") => infinite = v == "1",
                "version" => version = v,
                "orientation" => orientation ?= v.parse::<Orientation>(),
                "width" => width ?= v.parse::<u32>(),
                "height" => height ?= v.parse::<u32>(),
                "tilewidth" => tile_width ?= v.parse::<u32>(),
                "tileheight" => tile_height ?= v.parse::<u32>(),
            }
            ((colour, infinite), (version, orientation, width, height, tile_width, tile_height))
        );

        let infinite = infinite.unwrap_or(false);

        // We can only parse sequentally, but tilesets are guaranteed to appear before layers.
        // So we can pass in tileset data to layer construction without worrying about unfinished
        // data usage.
        let mut layers = Vec::new();
        let mut properties = HashMap::new();
        let mut tilesets = Vec::new();

        parse_tag!(parser, "map", {
            "tileset" => |attrs: Vec<OwnedAttribute>| {
                let res = Tileset::parse_xml_in_map(parser, &attrs, map_path,  reader, cache)?;
                match res.result_type {
                    EmbeddedParseResultType::ExternalReference { tileset_path } => {
                        let tileset = if let Some(ts) = cache.get_tileset(&tileset_path) {
                            ts
                        } else {
                            let tileset = Arc::new(Tileset::parse_xml(&tileset_path,  reader, cache)?);
                            cache.insert_tileset(tileset_path.clone(), tileset.clone());
                            tileset
                        };

                        tilesets.push(MapTilesetGid{first_gid: res.first_gid, tileset});
                    }
                    EmbeddedParseResultType::Embedded { tileset } => {
                        tilesets.push(MapTilesetGid{first_gid: res.first_gid, tileset: Arc::new(tileset)});
                    },
                };
                Ok(())
            },
            "layer" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::Tiles,
                    infinite,
                    map_path,
                    &tilesets,
                    None,
                    reader,
                    cache
                )?);
                Ok(())
            },
            "imagelayer" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::Image,
                    infinite,
                    map_path,
                    &tilesets,
                    None,
                    reader,
                    cache
                )?);
                Ok(())
            },
            "objectgroup" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::Objects,
                    infinite,
                    map_path,
                    &tilesets,
                    None,
                    reader,
                    cache
                )?);
                Ok(())
            },
            "group" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::Group,
                    infinite,
                    map_path,
                    &tilesets,
                    None,
                    reader,
                    cache
                )?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        // We do not need first GIDs any more
        let tilesets = tilesets.into_iter().map(|ts| ts.tileset).collect();

        Ok(Map {
            version: v,
            orientation: o,
            width: w,
            height: h,
            tile_width: tw,
            tile_height: th,
            tilesets,
            layers,
            properties,
            background_color: c,
            infinite,
        })
    }
}
