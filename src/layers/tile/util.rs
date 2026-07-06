use std::{convert::TryInto, io::Read};

use base64::Engine;

use crate::{
    CsvDecodingError, Error, LayerTileData, MapTilesetGid, Result, util::read_text_or_cdata,
};

pub(crate) fn parse_data_line<R: std::io::BufRead>(
    encoding: Option<String>,
    compression: Option<String>,
    elem: crate::util::XmlElement<'_, R>,
    tilesets: &[MapTilesetGid],
) -> Result<Vec<Option<LayerTileData>>> {
    let encoding_ref = encoding.as_deref();
    let compression_ref = compression.as_deref();
    read_text_or_cdata(elem, |text| match (encoding_ref, compression_ref) {
        (Some("csv"), None) => decode_csv(text, tilesets),

        (Some("base64"), None) => parse_base64(text).map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("zlib")) => parse_base64(text)
            .and_then(|data| process_decoder(Ok(flate2::bufread::ZlibDecoder::new(&data[..]))))
            .map(|v| convert_to_tiles(&v, tilesets)),
        (Some("base64"), Some("gzip")) => parse_base64(text)
            .and_then(|data| process_decoder(Ok(flate2::bufread::GzDecoder::new(&data[..]))))
            .map(|v| convert_to_tiles(&v, tilesets)),
        #[cfg(feature = "zstd")]
        (Some("base64"), Some("zstd")) => parse_base64(text)
            .and_then(|data| process_decoder(zstd::stream::read::Decoder::with_buffer(&data[..])))
            .map(|v| convert_to_tiles(&v, tilesets)),

        _ => Err(Error::InvalidEncodingFormat {
            encoding: encoding.clone(),
            compression: compression.clone(),
        }),
    })
}

fn parse_base64(text: &str) -> Result<Vec<u8>> {
    base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::general_purpose::PAD,
    )
    .decode(text.trim().as_bytes())
    .map_err(Error::Base64DecodingError)
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

fn decode_csv(text: &str, tilesets: &[MapTilesetGid]) -> Result<Vec<Option<LayerTileData>>> {
    let mut tiles = Vec::new();
    for v in text.split(',') {
        match v.trim().parse() {
            Ok(bits) => tiles.push(LayerTileData::from_bits(bits, tilesets)),
            Err(e) => {
                return Err(Error::CsvDecodingError(
                    CsvDecodingError::TileDataParseError(e),
                ));
            }
        }
    }
    Ok(tiles)
}

fn convert_to_tiles(data: &[u8], tilesets: &[MapTilesetGid]) -> Vec<Option<LayerTileData>> {
    data.chunks_exact(4)
        .map(|chunk| {
            let bits = u32::from_le_bytes(chunk.try_into().unwrap());
            LayerTileData::from_bits(bits, tilesets)
        })
        .collect()
}
