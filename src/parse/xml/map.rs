use std::io::BufReader;
use std::path::Path;

use quick_xml::Reader;

use crate::{Error, Map, ResourceCache, ResourceReader, Result, util::parse_root_element};

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
    parse_root_element(&mut parser, b"map", |elem| {
        Map::parse_xml(elem, path, reader, cache)
    })
}
