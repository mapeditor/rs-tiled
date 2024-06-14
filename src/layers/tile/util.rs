use std::{convert::TryInto, io::Read};

use base64::Engine;
use xml::reader::XmlEvent;

use crate::{util::XmlEventResult, CsvDecodingError, Error, LayerTileData, MapTilesetGid, Result};

pub(crate) fn parse_data_line(
    encoding: Option<String>,
    compression: Option<String>,
    parser: &mut impl Iterator<Item = XmlEventResult>,
    tilesets: &[MapTilesetGid],
) -> Result<Vec<Option<LayerTileData>>> {
    match (encoding.as_deref(), compression.as_deref()) {
        (Some("csv"), None) => decode_csv(parser, tilesets),

        (Some("base64"), None) => parse_base64(parser).map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("zlib")) => parse_base64(parser)
            .and_then(|data| process_decoder(Ok(flate2::bufread::ZlibDecoder::new(&data[..]))))
            .map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("gzip")) => parse_base64(parser)
            .and_then(|data| process_decoder(Ok(flate2::bufread::GzDecoder::new(&data[..]))))
            .map(|v| convert_to_tiles(&v, tilesets)),
        #[cfg(feature = "zstd")]
        (Some("base64"), Some("zstd")) => parse_base64(parser)
            .and_then(|data| process_decoder(zstd::stream::read::Decoder::with_buffer(&data[..])))
            .map(|v| convert_to_tiles(&v, tilesets)),

        _ => Err(Error::InvalidEncodingFormat {
            encoding,
            compression,
        }),
    }
}

fn parse_base64(parser: &mut impl Iterator<Item = XmlEventResult>) -> Result<Vec<u8>> {
    for next in parser {
        match next.map_err(Error::XmlDecodingError)? {
            XmlEvent::Characters(s) => {
                return base64::engine::GeneralPurpose::new(
                    &base64::alphabet::STANDARD,
                    base64::engine::general_purpose::PAD,
                )
                .decode(s.trim().as_bytes())
                .map_err(Error::Base64DecodingError);
            }
            XmlEvent::EndElement { name, .. } if name.local_name == "data" => {
                return Ok(Vec::new());
            }
            _ => {}
        }
    }
    Err(Error::PrematureEnd("Ran out of XML data".to_owned()))
}

fn process_decoder(decoder: std::io::Result<impl Read>) -> Result<Vec<u8>> {
    decoder
        .and_then(|mut decoder| {
            let mut data = Vec::new();
            decoder.read_to_end(&mut data)?;
            Ok(data)
        })
        .map_err(Error::DecompressingError)
}

fn decode_csv(
    parser: &mut impl Iterator<Item = XmlEventResult>,
    tilesets: &[MapTilesetGid],
) -> Result<Vec<Option<LayerTileData>>> {
    for next in parser {
        match next.map_err(Error::XmlDecodingError)? {
            XmlEvent::Characters(s) => {
                let mut tiles = Vec::new();
                for v in s.split(',') {
                    match v.trim().parse() {
                        Ok(bits) => tiles.push(LayerTileData::from_bits(bits, tilesets)),
                        Err(e) => {
                            return Err(Error::CsvDecodingError(
                                CsvDecodingError::TileDataParseError(e),
                            ))
                        }
                    }
                }
                return Ok(tiles);
            }
            XmlEvent::EndElement { name, .. } if name.local_name == "data" => {
                return Ok(Vec::new());
            }
            _ => {}
        }
    }
    Err(Error::PrematureEnd("Ran out of XML data".to_owned()))
}

fn convert_to_tiles(data: &[u8], tilesets: &[MapTilesetGid]) -> Vec<Option<LayerTileData>> {
    data.chunks_exact(4)
        .map(|chunk| {
            let bits = u32::from_le_bytes(chunk.try_into().unwrap());
            LayerTileData::from_bits(bits, tilesets)
        })
        .collect()
}
