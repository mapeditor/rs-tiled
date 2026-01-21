use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use quick_xml::Reader;

use crate::{
    util::{parse_root_element, parse_tag, XmlElement},
    EmbeddedParseResultType, Error, MapTilesetGid, ObjectData, ResourceCache, ResourceReader,
    Result, Tileset,
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
        let mut buf = Vec::new();
        parse_root_element(
            &mut template_parser,
            &mut buf,
            b"template",
            "Template Document ended before template element was parsed",
            |elem| Self::parse_external_template(elem, path, reader, cache),
        )
    }

    fn parse_external_template<R: std::io::BufRead>(
        elem: XmlElement<'_, R>,
        template_path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Arc<Template>> {
        let mut object = Option::None;
        let mut tileset = None;
        let mut tileset_gid: Vec<MapTilesetGid> = vec![];

        parse_tag!(elem, {
            "object" => |elem| {
                object = Some(ObjectData::new(elem, Some(&tileset_gid), tileset.clone(), template_path.parent().ok_or(Error::PathIsNotFile)?, reader, cache)?);
                Ok(())
            },
            "tileset" => |elem| {
                let res = Tileset::parse_xml_in_map(elem, template_path, reader, cache)?;
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
