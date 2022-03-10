use std::{io::Read, path::Path};

use xml::{reader::XmlEvent, EventReader};

use crate::{Error, Map, ResourceCache, Result};

pub fn parse_map(
    reader: impl Read,
    path: &Path,
    cache: &mut impl ResourceCache,
) -> Result<Map> {
    let mut parser = EventReader::new(reader);
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
