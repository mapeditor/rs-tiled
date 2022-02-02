/// Loops through the attributes once and pulls out the ones we ask it to. It
/// will check that the required ones are there. This could have been done with
/// attrs.find but that would be inefficient.
///
/// This is probably a really terrible way to do this. It does cut down on lines
/// though which is nice.
macro_rules! get_attrs {
    ($attrs:expr, optionals: [$(($oName:pat, $oVar:ident, $oMethod:expr)),* $(,)*],
     required: [$(($name:pat, $var:ident, $method:expr)),* $(,)*], $err:expr) => {
        {
            $(let mut $oVar = None;)*
            $(let mut $var = None;)*
            for attr in $attrs.iter() {
                match attr.name.local_name.as_ref() {
                    $($oName => $oVar = $oMethod(attr.value.clone()),)*
                    $($name => $var = $method(attr.value.clone()),)*
                    _ => {}
                }
            }
            if !(true $(&& $var.is_some())*) {
                return Err($err);
            }
            (($($oVar),*), ($($var.unwrap()),*))
        }
    }
}

/// Goes through the children of the tag and will call the correct function for
/// that child. Closes the tag.
macro_rules! parse_tag {
    ($parser:expr, $close_tag:expr, {$($open_tag:expr => $open_method:expr),* $(,)*}) => {
        loop {
            match $parser.next().map_err(TiledError::XmlDecodingError)? {
                xml::reader::XmlEvent::StartElement {name, attributes, ..} => {
                    if false {}
                    $(else if name.local_name == $open_tag {
                        match $open_method(attributes) {
                            Ok(()) => {},
                            Err(e) => return Err(e)
                        };
                    })*
                }
                xml::reader::XmlEvent::EndElement {name, ..} => {
                    if name.local_name == $close_tag {
                        break;
                    }
                }
                xml::reader::XmlEvent::EndDocument => return Err(TiledError::PrematureEnd("Document ended before we expected.".to_string())),
                _ => {}
            }
        }
    }
}

use std::io::{BufReader, Read};

pub(crate) use get_attrs;
pub(crate) use parse_tag;
use xml::{reader::XmlEvent, EventReader};

use crate::{animation::Frame, error::TiledError, LayerTileData};

// TODO: Move to animation module
pub(crate) fn parse_animation<R: Read>(
    parser: &mut EventReader<R>,
) -> Result<Vec<Frame>, TiledError> {
    let mut animation = Vec::new();
    parse_tag!(parser, "animation", {
        "frame" => |attrs| {
            animation.push(Frame::new(attrs)?);
            Ok(())
        },
    });
    Ok(animation)
}

// TODO: Move to layers module
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

pub(crate) fn parse_base64<R: Read>(parser: &mut EventReader<R>) -> Result<Vec<u8>, TiledError> {
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

pub(crate) fn decode_zlib(data: Vec<u8>) -> Result<Vec<u8>, TiledError> {
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

pub(crate) fn decode_gzip(data: Vec<u8>) -> Result<Vec<u8>, TiledError> {
    use libflate::gzip::Decoder;
    let mut zd =
        Decoder::new(BufReader::new(&data[..])).map_err(|e| TiledError::DecompressingError(e))?;

    let mut data = Vec::new();
    zd.read_to_end(&mut data)
        .map_err(|e| TiledError::DecompressingError(e))?;
    Ok(data)
}

#[cfg(feature = "zstd")]
pub(crate) fn decode_zstd(data: Vec<u8>) -> Result<Vec<u8>, TiledError> {
    use std::io::Cursor;
    use zstd::stream::read::Decoder;

    let buff = Cursor::new(&data);
    let mut zd = Decoder::with_buffer(buff).map_err(|e| TiledError::DecompressingError(e))?;

    let mut data = Vec::new();
    zd.read_to_end(&mut data)
        .map_err(|e| TiledError::DecompressingError(e))?;
    Ok(data)
}

// TODO: Move to layers module
pub(crate) fn decode_csv<R: Read>(
    parser: &mut EventReader<R>,
) -> Result<Vec<LayerTileData>, TiledError> {
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

// TODO: Move to layers module
pub(crate) fn convert_to_tiles(all: &Vec<u8>) -> Vec<LayerTileData> {
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
