use std::{path::Path, sync::Arc};

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    parse::common::tileset::EmbeddedParseResultType,
    util::{parse_tag, XmlEventResult},
    Error, MapTilesetGid, ObjectData, ResourceCache, ResourceReader, Result, Template, Tileset,
};

impl Template {
    pub(crate) fn parse_template(
        path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Arc<Template>> {
        // Open the template file
        let file = reader
            .read_from(path)
            .map_err(|err| Error::ResourceLoadingError {
                path: path.to_owned(),
                err: Box::new(err),
            })?;

        let mut template_parser = EventReader::new(file);
        loop {
            match template_parser.next().map_err(Error::XmlDecodingError)? {
                XmlEvent::StartElement {
                    name,
                    attributes: _,
                    ..
                } if name.local_name == "template" => {
                    let template = Self::parse_external_template(
                        &mut template_parser.into_iter(),
                        path,
                        reader,
                        cache,
                    )?;
                    return Ok(template);
                }
                XmlEvent::EndDocument => {
                    return Err(Error::PrematureEnd(
                        "Template Document ended before template element was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }

    fn parse_external_template(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        template_path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Arc<Template>> {
        let mut object = Option::None;
        let mut tileset = None;
        let mut tileset_gid: Vec<MapTilesetGid> = vec![];

        parse_tag!(parser, "template", {
            "object" => |attrs| {
                object = Some(ObjectData::new(parser, attrs, Some(&tileset_gid), tileset.clone(), template_path.parent().ok_or(Error::PathIsNotFile)?, reader, cache)?);
                Ok(())
            },
            "tileset" => |attrs: Vec<OwnedAttribute>| {
                let res = Tileset::parse_xml_in_map(parser, &attrs, template_path, reader, cache)?;
                match res.result_type {
                    EmbeddedParseResultType::ExternalReference { tileset_path } => {
                        tileset = Some(if let Some(ts) = cache.get_tileset(&tileset_path) {
                            ts
                        } else {
                            let tileset = Arc::new(Tileset::parse_xml(&tileset_path, reader, cache)?);
                            cache.insert_tileset(tileset_path.clone(), tileset.clone());
                            tileset
                        });
                    }
                    EmbeddedParseResultType::Embedded { tileset: embedded_tileset } => {
                        tileset = Some(Arc::new(embedded_tileset));
                    },
                };
                tileset_gid.push(MapTilesetGid {
                    tileset: tileset.clone().unwrap(),
                    first_gid: res.first_gid,
                });
                Ok(())
            },
        });

        let object = object.ok_or(Error::TemplateHasNoObject)?;

        Ok(Arc::new(Template { tileset, object }))
    }
}
