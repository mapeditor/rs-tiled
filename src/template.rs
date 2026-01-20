use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{
    util::parse_tag,
    EmbeddedParseResultType, Error, MapTilesetGid, ObjectData, ResourceCache,
    ResourceReader, Result, Tileset,
};

/// A template, consisting of an object and a tileset
///
/// Templates define a tileset and object data to use for an object that can be shared between multiple objects and
/// maps.
#[derive(Clone, Debug)]
pub struct Template {
    /// The path first used in a [`ResourceReader`] to load this template.
    pub source: PathBuf,
    /// The tileset this template contains a reference to
    pub tileset: Option<Arc<Tileset>>,
    /// The object data for this template
    pub object: ObjectData,
}

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

        let mut template_parser = Reader::from_reader(BufReader::new(file));
        template_parser.expand_empty_elements(true);
        let mut buf = Vec::new();
        loop {
            match template_parser
                .read_event_into(&mut buf)
                .map_err(Error::XmlDecodingError)?
            {
                Event::Start(ref e) if e.local_name().as_ref() == b"template" => {
                    buf.clear();
                    let template = Self::parse_external_template(
                        &mut template_parser,
                        &mut buf,
                        path,
                        reader,
                        cache,
                    )?;
                    return Ok(template);
                }
                Event::Eof => {
                    return Err(Error::PrematureEnd(
                        "Template Document ended before template element was parsed".to_string(),
                    ))
                }
                _ => {}
            }
            buf.clear();
        }
    }

    fn parse_external_template(
        xml_reader: &mut Reader<impl std::io::BufRead>,
        buf: &mut Vec<u8>,
        template_path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Arc<Template>> {
        let mut object = Option::None;
        let mut tileset = None;
        let mut tileset_gid: Vec<MapTilesetGid> = vec![];

        buf.clear();
        parse_tag!(xml_reader, buf, "template", {
            "object" => |attrs| {
                object = Some(ObjectData::new(xml_reader, buf, attrs, Some(&tileset_gid), tileset.clone(), template_path.parent().ok_or(Error::PathIsNotFile)?, reader, cache)?);
                Ok(())
            },
            "tileset" => |attrs| {
                let res = Tileset::parse_xml_in_map(xml_reader, buf, attrs, template_path, reader, cache)?;
                match res.result_type {
                    EmbeddedParseResultType::ExternalReference { tileset_path } => {
                        tileset = Some(if let Some(ts) = cache.get_tileset(&tileset_path) {
                            ts
                        } else {
                            let tileset = Arc::new(crate::parse::xml::parse_tileset(&tileset_path, reader, cache)?);
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

        Ok(Arc::new(Template {
            source: template_path.to_owned(),
            tileset,
            object,
        }))
    }
}
