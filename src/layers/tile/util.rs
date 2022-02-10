use std::io::{BufReader, Read};

use xml::reader::XmlEvent;

use crate::{util::XmlEventResult, LayerTileData, MapTilesetGid, TiledError};

pub(crate) fn parse_data_line(
    encoding: Option<String>,
    compression: Option<String>,
    parser: &mut impl Iterator<Item = XmlEventResult>,
    tilesets: &[MapTilesetGid],
) -> Result<Vec<Option<LayerTileData>>, TiledError> {
    match (encoding.as_deref(), compression.as_deref()) {
        (None, None) => {
            return Err(TiledError::InvalidEncodingFormat {
                encoding,
                compression,
            })
        }
        (Some("base64"), None) => {
            return parse_base64(parser).map(|v| convert_to_tiles(&v, tilesets))
        }
        (Some("csv"), None) => return decode_csv(parser, tilesets),
        (Some(_), None) => {
            return Err(TiledError::InvalidEncodingFormat {
                encoding,
                compression,
            })
        }
        (Some(e), Some(c)) => match (e, c) {
            ("base64", "zlib") => {
                return parse_base64(parser)
                    .and_then(decode_zlib)
                    .map(|v| convert_to_tiles(&v, tilesets))
            }
            ("base64", "gzip") => {
                return parse_base64(parser)
                    .and_then(decode_gzip)
                    .map(|v| convert_to_tiles(&v, tilesets))
            }
            #[cfg(feature = "zstd")]
            ("base64", "zstd") => {
                return parse_base64(parser)
                    .and_then(decode_zstd)
                    .map(|v| convert_to_tiles(&v, tilesets))
            }
            _ => {
                return Err(TiledError::InvalidEncodingFormat {
                    encoding,
                    compression,
                })
            }
        },
        _ => {
            return Err(TiledError::InvalidEncodingFormat {
                encoding,
                compression,
            })
        }
    };
}

fn parse_base64(parser: &mut impl Iterator<Item = XmlEventResult>) -> Result<Vec<u8>, TiledError> {
    while let Some(next) = parser.next() {
        match next.map_err(TiledError::XmlDecodingError)? {
            XmlEvent::Characters(s) => {
                return base64::decode(s.trim().as_bytes()).map_err(TiledError::Base64DecodingError)
            }
            XmlEvent::EndElement { name, .. } => {
                if name.local_name == "data" {
                    return Ok(Vec::new());
                }
            }
            _ => {}
        }
    }
    Err(TiledError::PrematureEnd("Ran out of XML data".to_owned()))
}

fn decode_zlib(data: Vec<u8>) -> Result<Vec<u8>, TiledError> {
    use libflate::zlib::Decoder;
    let mut zd =
        Decoder::new(BufReader::new(&data[..])).map_err(|e| TiledError::DecompressingError(e))?;
    let mut data = Vec::new();
    match zd.read_to_end(&mut data) {
        Ok(_v) => {}
        Err(e) => return Err(TiledError::DecompressingError(e)),
    }
    Ok(data)
}

fn decode_gzip(data: Vec<u8>) -> Result<Vec<u8>, TiledError> {
    use libflate::gzip::Decoder;
    let mut zd =
        Decoder::new(BufReader::new(&data[..])).map_err(|e| TiledError::DecompressingError(e))?;

    let mut data = Vec::new();
    zd.read_to_end(&mut data)
        .map_err(|e| TiledError::DecompressingError(e))?;
    Ok(data)
}

fn decode_zstd(data: Vec<u8>) -> Result<Vec<u8>, TiledError> {
    use std::io::Cursor;
    use zstd::stream::read::Decoder;

    let buff = Cursor::new(&data);
    let mut zd = Decoder::with_buffer(buff).map_err(|e| TiledError::DecompressingError(e))?;

    let mut data = Vec::new();
    zd.read_to_end(&mut data)
        .map_err(|e| TiledError::DecompressingError(e))?;
    Ok(data)
}

fn decode_csv(
    parser: &mut impl Iterator<Item = XmlEventResult>,
    tilesets: &[MapTilesetGid],
) -> Result<Vec<Option<LayerTileData>>, TiledError> {
    while let Some(next) = parser.next() {
        match next.map_err(TiledError::XmlDecodingError)? {
            XmlEvent::Characters(s) => {
                let tiles = s
                    .split(',')
                    .map(|v| v.trim().parse().unwrap())
                    .map(|bits| LayerTileData::from_bits(bits, tilesets))
                    .collect();
                return Ok(tiles);
            }
            XmlEvent::EndElement { name, .. } => {
                if name.local_name == "data" {
                    return Ok(Vec::new());
                }
            }
            _ => {}
        }
    }
    Err(TiledError::PrematureEnd("Ran out of XML data".to_owned()))
}

fn convert_to_tiles(all: &Vec<u8>, tilesets: &[MapTilesetGid]) -> Vec<Option<LayerTileData>> {
    let mut data = Vec::new();
    for chunk in all.chunks_exact(4) {
        let n = chunk[0] as u32
            + ((chunk[1] as u32) << 8)
            + ((chunk[2] as u32) << 16)
            + ((chunk[3] as u32) << 24);
        data.push(LayerTileData::from_bits(n, tilesets));
    }
    data
}
