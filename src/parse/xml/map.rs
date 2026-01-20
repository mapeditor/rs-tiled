use std::io::BufReader;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{Error, Map, ResourceCache, ResourceReader, Result};

pub fn parse_map(
    path: &Path,
    reader: &mut impl ResourceReader,
    cache: &mut impl ResourceCache,
) -> Result<Map> {
    let file = reader
        .read_from(path)
        .map_err(|err| Error::ResourceLoadingError {
            path: path.to_owned(),
            err: Box::new(err),
        })?;
    let mut parser = Reader::from_reader(BufReader::new(file));
    parser.expand_empty_elements(true);
    let mut buf = Vec::new();
    let mut event_buf = Vec::new();
    loop {
        match parser
            .read_event_into(&mut event_buf)
            .map_err(Error::XmlDecodingError)?
        {
            Event::Start(e) if e.local_name().as_ref() == b"map" => {
                return Map::parse_xml(
                    &mut parser,
                    &mut buf,
                    e.into_owned(),
                    path,
                    reader,
                    cache,
                );
            }
            Event::Eof => {
                return Err(Error::PrematureEnd(
                    "Document ended before map was parsed".to_string(),
                ))
            }
            _ => {}
        }
        event_buf.clear();
    }
}
