use std::{convert::TryInto, io::Read};

use base64::Engine;
use quick_xml::events::Event;

use crate::{CsvDecodingError, Error, LayerTileData, MapTilesetGid, Result};

pub(crate) fn parse_data_line(
    encoding: Option<String>,
    compression: Option<String>,
    reader: &mut quick_xml::Reader<impl std::io::BufRead>,
    buf: &mut Vec<u8>,
    tilesets: &[MapTilesetGid],
) -> Result<Vec<Option<LayerTileData>>> {
    match (encoding.as_deref(), compression.as_deref()) {
        (Some("csv"), None) => decode_csv(reader, buf, tilesets),

        (Some("base64"), None) => parse_base64(reader, buf).map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("zlib")) => parse_base64(reader, buf)
            .and_then(|data| process_decoder(Ok(flate2::bufread::ZlibDecoder::new(&data[..]))))
            .map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("gzip")) => parse_base64(reader, buf)
            .and_then(|data| process_decoder(Ok(flate2::bufread::GzDecoder::new(&data[..]))))
            .map(|v| convert_to_tiles(&v, tilesets)),
        #[cfg(feature = "zstd")]
        (Some("base64"), Some("zstd")) => parse_base64(reader, buf)
            .and_then(|data| process_decoder(zstd::stream::read::Decoder::with_buffer(&data[..])))
            .map(|v| convert_to_tiles(&v, tilesets)),

        _ => Err(Error::InvalidEncodingFormat {
            encoding,
            compression,
        }),
    }
}

fn parse_base64(
    reader: &mut quick_xml::Reader<impl std::io::BufRead>,
    buf: &mut Vec<u8>,
) -> Result<Vec<u8>> {
    loop {
        match reader.read_event_into(buf).map_err(Error::XmlDecodingError)? {
            Event::Text(e) => {
                let text = e.unescape().map_err(Error::XmlDecodingError)?.into_owned();
                buf.clear();
                return base64::engine::GeneralPurpose::new(
                    &base64::alphabet::STANDARD,
                    base64::engine::general_purpose::PAD,
                )
                .decode(text.trim().as_bytes())
                .map_err(Error::Base64DecodingError);
            }
            Event::CData(e) => {
                let text = String::from_utf8_lossy(e.as_ref()).into_owned();
                buf.clear();
                return base64::engine::GeneralPurpose::new(
                    &base64::alphabet::STANDARD,
                    base64::engine::general_purpose::PAD,
                )
                .decode(text.trim().as_bytes())
                .map_err(Error::Base64DecodingError);
            }
            Event::End(ref e) if e.local_name().as_ref() == b"data" => {
                buf.clear();
                return Ok(Vec::new());
            }
            Event::Eof => {
                return Err(Error::PrematureEnd("Ran out of XML data".to_owned()));
            }
            _ => {}
        }
        buf.clear();
    }
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
    reader: &mut quick_xml::Reader<impl std::io::BufRead>,
    buf: &mut Vec<u8>,
    tilesets: &[MapTilesetGid],
) -> Result<Vec<Option<LayerTileData>>> {
    loop {
        match reader.read_event_into(buf).map_err(Error::XmlDecodingError)? {
            Event::Text(e) => {
                let text = e.unescape().map_err(Error::XmlDecodingError)?.into_owned();
                buf.clear();
                let mut tiles = Vec::new();
                for v in text.split(',') {
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
            Event::CData(e) => {
                let text = String::from_utf8_lossy(e.as_ref()).into_owned();
                buf.clear();
                let mut tiles = Vec::new();
                for v in text.split(',') {
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
            Event::End(ref e) if e.local_name().as_ref() == b"data" => {
                buf.clear();
                return Ok(Vec::new());
            }
            Event::Eof => {
                return Err(Error::PrematureEnd("Ran out of XML data".to_owned()));
            }
            _ => {}
        }
        buf.clear();
    }
}

fn convert_to_tiles(data: &[u8], tilesets: &[MapTilesetGid]) -> Vec<Option<LayerTileData>> {
    data.chunks_exact(4)
        .map(|chunk| {
            let bits = u32::from_le_bytes(chunk.try_into().unwrap());
            LayerTileData::from_bits(bits, tilesets)
        })
        .collect()
}
