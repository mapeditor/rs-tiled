use std::{path::Path, sync::Arc};

use xml::{reader::XmlEvent, EventReader};

use crate::{Error, ResourceCache, ResourceReader, Result, Tileset};

pub fn parse_tileset(
    path: &Path,
    reader: &mut impl ResourceReader,
    cache: &mut impl ResourceCache,
) -> Result<Tileset> {
    parse_for_tileset(path, None, reader, cache)
}

/// Parse a tileset from a reader, but updates a list of templates
///
/// Used by Maps and Templates which require a state for managing the template list
pub(crate) fn parse_for_tileset(
    path: impl AsRef<Path>,
    for_tileset: Option<Arc<Tileset>>,
    reader: &mut impl ResourceReader,
    cache: &mut impl ResourceCache,
) -> Result<Tileset> {
    let path = path.as_ref();
    let mut tileset_parser =
        EventReader::new(
            reader
                .read_from(path)
                .map_err(|err| Error::ResourceLoadingError {
                    path: path.to_owned(),
                    err: Box::new(err),
                })?,
        );
    loop {
        match tileset_parser.next().map_err(Error::XmlDecodingError)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } if name.local_name == "tileset" => {
                return Tileset::parse_external_tileset(
                    &mut tileset_parser.into_iter(),
                    &attributes,
                    path.as_ref(),
                    for_tileset,
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
