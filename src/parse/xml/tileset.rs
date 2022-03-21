use std::path::Path;

use xml::{reader::XmlEvent, EventReader};

use crate::{Error, ResourceReader, Result, Tileset};

pub fn parse_tileset(path: &Path, reader: &mut impl ResourceReader) -> Result<Tileset> {
    let mut tileset_parser =
        EventReader::new(
            reader
                .read_from(path)
                .map_err(|err| Error::CouldNotOpenFile {
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
                    path,
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
