use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;
use xml::EventReader;

use crate::error::TiledError;
use crate::{util::*, EmbeddedParseResultType, MapTilesetGid, ObjectData, ResourceCache, Tileset};

/// A template, consisting of an object and a tileset
///
/// Templates define a tileset and object data to use for an object that can be shared between multiple objects and
/// maps.
#[derive(Clone, Debug)]
pub struct Template {
    /// The tileset this template contains a reference to
    pub tileset: Option<Arc<Tileset>>,
    /// The object data for this template
    pub object: Option<ObjectData>,
}

impl Template {
    pub(crate) fn parse_template(
        path: &Path,
        cache: &mut impl ResourceCache,
    ) -> Result<Arc<Template>, TiledError> {
        // Check the cache to see if this template exists
        if let Some(templ) = cache.get_template(path) {
            return Ok(templ);
        }

        // Open the template file
        let file = File::open(&path).map_err(|err| TiledError::CouldNotOpenFile {
            path: path.to_path_buf(),
            err,
        })?;

        let mut template_parser = EventReader::new(file);
        loop {
            match template_parser
                .next()
                .map_err(TiledError::XmlDecodingError)?
            {
                XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == "template" => {
                    let template = Self::parse_external_template(
                        &mut template_parser.into_iter(),
                        &attributes,
                        path,
                        cache,
                    )?;

                    // Insert it into the cache
                    cache.insert_template(path.to_path_buf(), template.clone());
                    return Ok(template);
                }
                XmlEvent::EndDocument => {
                    return Err(TiledError::PrematureEnd(
                        "Template Document ended before template element was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }

    fn parse_external_template(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        _attrs: &Vec<OwnedAttribute>,
        template_path: &Path,
        cache: &mut impl ResourceCache,
    ) -> Result<Arc<Template>, TiledError> {
        let mut object = Option::None;
        let mut tileset = None;
        let mut tileset_gid: Vec<MapTilesetGid> = vec![];

        parse_tag!(parser, "template", {
            "object" => |attrs| {
                object = Some(ObjectData::new(parser, attrs, Some(&tileset_gid), tileset.clone(), template_path, cache)?);
                Ok(())
            },
            "tileset" => |attrs| {
                let res = Tileset::parse_xml_in_map(parser, attrs, template_path, tileset.clone(), cache)?;
                match res.result_type {
                    EmbeddedParseResultType::ExternalReference { tileset_path } => {
                        tileset = Some(if let Some(ts) = cache.get_tileset(&tileset_path) {
                            ts
                        } else {
                            let file = File::open(&tileset_path).map_err(|err| TiledError::CouldNotOpenFile{path: tileset_path.clone(), err })?;
                            let tileset = Arc::new(Tileset::parse_with_template_list(file, &tileset_path, cache, None)?);
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

        Ok(Arc::new(Template { tileset, object }))
    }
}
