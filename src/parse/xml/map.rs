use std::io::BufReader;
use std::path::Path;

use quick_xml::Reader;

use crate::{util::parse_root_element, Error, Map, ResourceCache, ResourceReader, Result};

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
    let mut buf = Vec::new();
    parse_root_element(
        &mut parser,
        &mut buf,
        b"map",
        "Document ended before map was parsed",
        |elem| Map::parse_xml(elem, path, reader, cache),
    )
}
