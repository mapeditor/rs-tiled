use std::io::{BufReader, Read};

use xml::{reader::XmlEvent, EventReader};

use crate::{LayerTileData, TiledError};

pub(crate) fn parse_data_line<R: Read>(
    encoding: Option<String>,
    compression: Option<String>,
    parser: &mut EventReader<R>,
) -> Result<Vec<LayerTileData>, TiledError> {
    match (encoding, compression) {
        (None, None) => {
            return Err(TiledError::Other(
                "XML format is currently not supported".to_string(),
            ))
        }
        (Some(e), None) => match e.as_ref() {
            "base64" => return parse_base64(parser).map(|v| convert_to_tiles(&v)),
            "csv" => return decode_csv(parser),
            e => return Err(TiledError::Other(format!("Unknown encoding format {}", e))),
        },
        (Some(e), Some(c)) => match (e.as_ref(), c.as_ref()) {
            ("base64", "zlib") => {
                return parse_base64(parser)
                    .and_then(decode_zlib)
                    .map(|v| convert_to_tiles(&v))
            }
            ("base64", "gzip") => {
                return parse_base64(parser)
                    .and_then(decode_gzip)
                    .map(|v| convert_to_tiles(&v))
            }
            #[cfg(feature = "zstd")]
            ("base64", "zstd") => {
                return parse_base64(parser)
                    .and_then(decode_zstd)
                    .map(|v| convert_to_tiles(&v))
            }
            (e, c) => {
                return Err(TiledError::Other(format!(
                    "Unknown combination of {} encoding and {} compression",
                    e, c
                )))
            }
        },
        _ => return Err(TiledError::Other("Missing encoding format".to_string())),
    };
}

fn parse_base64<R: Read>(parser: &mut EventReader<R>) -> Result<Vec<u8>, TiledError> {
    loop {
        match parser.next().map_err(TiledError::XmlDecodingError)? {
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

fn decode_csv<R: Read>(parser: &mut EventReader<R>) -> Result<Vec<LayerTileData>, TiledError> {
    loop {
        match parser.next().map_err(TiledError::XmlDecodingError)? {
            XmlEvent::Characters(s) => {
                let tiles = s
                    .split(',')
                    .map(|v| v.trim().parse().unwrap())
                    .map(LayerTileData::from_bits)
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
}

fn convert_to_tiles(all: &Vec<u8>) -> Vec<LayerTileData> {
    let mut data = Vec::new();
    for chunk in all.chunks_exact(4) {
        let n = chunk[0] as u32
            + ((chunk[1] as u32) << 8)
            + ((chunk[2] as u32) << 16)
            + ((chunk[3] as u32) << 24);
        data.push(LayerTileData::from_bits(n));
    }
    data
}
