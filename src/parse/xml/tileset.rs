use std::{io::Read, path::Path, sync::Arc};

use xml::{reader::XmlEvent, EventReader};

use crate::{Error, ResourceCache, Result, Tileset};

pub fn parse_tileset<R: Read>(
    reader: R,
    path: &Path,
    cache: &mut impl ResourceCache,
) -> Result<Tileset> {
    parse_for_tileset(reader, path, None, cache)
}

/// Parse a tileset from a reader, but updates a list of templates
///
/// Used by Maps and Templates which require a state for managing the template list
pub(crate) fn parse_for_tileset<R: Read>(
    reader: R,
    path: impl AsRef<Path>,
    for_tileset: Option<Arc<Tileset>>,
    cache: &mut impl ResourceCache,
) -> Result<Tileset> {
    let mut tileset_parser = EventReader::new(reader);
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
