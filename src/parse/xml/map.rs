use std::path::Path;

use xml::{reader::XmlEvent, EventReader};

use crate::{Error, Map, ResourceCache, ResourceReader, Result};

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
                        cache,
                        reader,
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
