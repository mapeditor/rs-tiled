use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;
use xml::EventReader;

use crate::error::TiledError;
use crate::{util::*, EmbeddedParseResultType, MapTilesetGid, ObjectData, ResourceCache, Tileset};

/// A template, consisting of an object and a tileset
///
/// The path is the path to the template file; it is used to identify them uniquely, and skip parsing the file multiple
/// times.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Template {
    pub path: PathBuf,
    pub tileset: Option<Arc<Tileset>>,
    pub object: Option<ObjectData>,
}

impl Template {
    pub(crate) fn parse_and_append_template(
        path: &Path,
        templates: &mut Vec<Template>,
        cache: &mut impl ResourceCache,
    ) -> Result<usize, TiledError> {
        // First, check if this template already exists in our list of templates (it will be uniquely identified by its
        // path).
        if let Some((id, _)) = templates
            .iter()
            .enumerate()
            .find(|(_, template)| template.path == path)
        {
            return Ok(id);
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
                    return Self::parse_and_append_external_template(
                        &mut template_parser.into_iter(),
                        &attributes,
                        templates,
                        path,
                        cache,
                    );
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

    fn parse_and_append_external_template(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        _attrs: &Vec<OwnedAttribute>,
        templates: &mut Vec<Template>,
        template_path: &Path,
        cache: &mut impl ResourceCache,
    ) -> Result<usize, TiledError> {
        let mut object = Option::None;

        let template_id = templates.len();
        templates.push(Template {
            path: template_path.to_path_buf(),
            tileset: None,
            object: None,
        });

        let mut tilesets = vec![];

        parse_tag!(parser, "template", {
            "object" => |attrs| {
                object = Some(ObjectData::new(parser, attrs, Some(&tilesets), templates, Some(template_id), template_path, cache)?);
                Ok(())
            },
            "tileset" => |attrs| {
                let res = Tileset::parse_xml_in_map(parser, attrs, template_path, templates, Some(template_id), cache)?;
                match res.result_type {
                    EmbeddedParseResultType::ExternalReference { tileset_path } => {
                        let tileset = if let Some(ts) = cache.get_tileset(&tileset_path) {
                            ts
                        } else {
                            let file = File::open(&tileset_path).map_err(|err| TiledError::CouldNotOpenFile{path: tileset_path.clone(), err })?;
                            let tileset = Arc::new(Tileset::parse_with_template_list(file, &tileset_path, cache, templates, Some(template_id))?);
                            cache.insert_tileset(tileset_path.clone(), tileset.clone());
                            tileset
                        };
                        tilesets.push(MapTilesetGid{first_gid: res.first_gid, tileset: tileset.clone()});
                    }
                    EmbeddedParseResultType::Embedded { tileset } => {
                        let arc = Arc::new(tileset);
                        tilesets.push(MapTilesetGid{first_gid: res.first_gid, tileset: arc.clone()});
                    },
                };
                Ok(())
            },
        });

        templates[template_id].tileset = tilesets.drain(..).next().map(|x| x.tileset);
        templates[template_id].object = object;
        Ok(template_id)
    }
}
