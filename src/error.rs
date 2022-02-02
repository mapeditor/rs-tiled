use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum ParseTileError {
    ColorError,
    OrientationError,
}

/// Errors which occured when parsing the file
#[derive(Debug)]
pub enum TiledError {
    /// A attribute was missing, had the wrong type of wasn't formated
    /// correctly.
    MalformedAttributes(String),
    /// An error occured when decompressing using the
    /// [flate2](https://github.com/alexcrichton/flate2-rs) crate.
    DecompressingError(std::io::Error),
    Base64DecodingError(base64::DecodeError),
    XmlDecodingError(xml::reader::Error),
    PrematureEnd(String),
    /// The path given is invalid because it isn't contained in any folder.
    InvalidPath,
    Other(String),
}

impl fmt::Display for TiledError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            TiledError::MalformedAttributes(ref s) => write!(fmt, "{}", s),
            TiledError::DecompressingError(ref e) => write!(fmt, "{}", e),
            TiledError::Base64DecodingError(ref e) => write!(fmt, "{}", e),
            TiledError::XmlDecodingError(ref e) => write!(fmt, "{}", e),
            TiledError::PrematureEnd(ref e) => write!(fmt, "{}", e),
            TiledError::InvalidPath => {
                write!(
                    fmt,
                    "The map path given is invalid because it isn't contained in any folder."
                )
            }
            TiledError::Other(ref s) => write!(fmt, "{}", s),
        }
    }
}

// This is a skeleton implementation, which should probably be extended in the future.
impl std::error::Error for TiledError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            TiledError::DecompressingError(ref e) => Some(e as &dyn std::error::Error),
            TiledError::Base64DecodingError(ref e) => Some(e as &dyn std::error::Error),
            TiledError::XmlDecodingError(ref e) => Some(e as &dyn std::error::Error),
            _ => None,
        }
    }
}
