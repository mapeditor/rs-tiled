use std::io::Read;

use xml::reader::XmlEvent;

use crate::{util::XmlEventResult, LayerTileData, MapTilesetGid, TiledError};

pub(crate) fn parse_data_line(
    encoding: Option<String>,
    compression: Option<String>,
    parser: &mut impl Iterator<Item = XmlEventResult>,
    tilesets: &[MapTilesetGid],
) -> Result<Vec<Option<LayerTileData>>, TiledError> {
    match (encoding.as_deref(), compression.as_deref()) {
        (Some("csv"), None) => decode_csv(parser, tilesets),

        (Some("base64"), None) => parse_base64(parser).map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("zlib")) => parse_base64(parser)
            .map(|data| std::io::Cursor::new(data))
            .and_then(|reader| process_decoder(libflate::zlib::Decoder::new(reader)))
            .map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("gzip")) => parse_base64(parser)
            .map(|data| std::io::Cursor::new(data))
            .and_then(|reader| process_decoder(libflate::gzip::Decoder::new(reader)))
            .map(|v| convert_to_tiles(&v, tilesets)),
        #[cfg(feature = "zstd")]
        (Some("base64"), Some("zstd")) => parse_base64(parser)
            .map(|data| std::io::Cursor::new(data))
            .and_then(|reader| process_decoder(zstd::stream::read::Decoder::with_buffer(reader)))
            .map(|v| convert_to_tiles(&v, tilesets)),

        _ => Err(TiledError::InvalidEncodingFormat {
            encoding,
            compression,
        }),
    }
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

fn process_decoder(decoder: std::io::Result<impl Read>) -> Result<Vec<u8>, TiledError> {
    decoder
        .and_then(|mut decoder| {
            let mut data = Vec::new();
            decoder.read_to_end(&mut data)?;
            Ok(data)
        })
        .map_err(|e| TiledError::DecompressingError(e))
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
    all.chunks_exact(4)
        .map(|chunk| {
            let bits = chunk[0] as u32
                + ((chunk[1] as u32) << 8)
                + ((chunk[2] as u32) << 16)
                + ((chunk[3] as u32) << 24);
            LayerTileData::from_bits(bits, tilesets)
        })
        .collect()
}
