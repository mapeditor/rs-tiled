use std::io::BufReader;
use std::path::Path;

use quick_xml::Reader;

use crate::{util::parse_root_element, Error, ResourceCache, ResourceReader, Result, Tileset};

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
    parse_root_element(
        &mut tileset_parser,
        b"tileset",
        "Tileset Document ended before map was parsed",
        |elem| Tileset::parse_external_tileset(elem, path, reader, cache),
    )
}
