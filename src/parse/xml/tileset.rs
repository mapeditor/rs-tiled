use std::io::BufReader;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{Error, ResourceCache, ResourceReader, Result, Tileset};

pub fn parse_tileset(
    path: &Path,
    reader: &mut impl ResourceReader,
    cache: &mut impl ResourceCache,
) -> Result<Tileset> {
    let file = reader
        .read_from(path)
        .map_err(|err| Error::ResourceLoadingError {
            path: path.to_owned(),
            err: Box::new(err),
        })?;
    let mut tileset_parser = Reader::from_reader(BufReader::new(file));
    tileset_parser.expand_empty_elements(true);
    let mut buf = Vec::new();
    let mut event_buf = Vec::new();
    loop {
        match tileset_parser
            .read_event_into(&mut event_buf)
            .map_err(Error::XmlDecodingError)?
        {
            Event::Start(e) if e.local_name().as_ref() == b"tileset" => {
                return Tileset::parse_external_tileset(
                    &mut tileset_parser,
                    &mut buf,
                    e.into_owned(),
                    path,
                    reader,
                    cache,
                );
            }
            Event::Eof => {
                return Err(Error::PrematureEnd(
                    "Tileset Document ended before map was parsed".to_string(),
                ))
            }
            _ => {}
        }
        event_buf.clear();
    }
}
