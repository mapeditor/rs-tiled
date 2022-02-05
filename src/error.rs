use std::{fmt, path::PathBuf};

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
    PathIsNotFile,
    CouldNotOpenFile {
        path: PathBuf,
        err: std::io::Error,
    },
    /// There was an invalid tile in the map parsed.
    InvalidTileFound,
    Other(String),
}

impl fmt::Display for TiledError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            TiledError::MalformedAttributes(s) => write!(fmt, "{}", s),
            TiledError::DecompressingError(e) => write!(fmt, "{}", e),
            TiledError::Base64DecodingError(e) => write!(fmt, "{}", e),
            TiledError::XmlDecodingError(e) => write!(fmt, "{}", e),
            TiledError::PrematureEnd(e) => write!(fmt, "{}", e),
            TiledError::PathIsNotFile => {
                write!(
                    fmt,
                    "The path given is invalid because it isn't contained in any folder."
                )
            }
            TiledError::CouldNotOpenFile { path, err } => {
                write!(
                    fmt,
                    "Could not open '{}'. Error: {}",
                    path.to_string_lossy(),
                    err
                )
            }
            TiledError::InvalidTileFound => write!(fmt, "Invalid tile found in map being parsed"),
            TiledError::Other(s) => write!(fmt, "{}", s),
        }
    }
}

// This is a skeleton implementation, which should probably be extended in the future.
impl std::error::Error for TiledError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TiledError::DecompressingError(e) => Some(e as &dyn std::error::Error),
            TiledError::Base64DecodingError(e) => Some(e as &dyn std::error::Error),
            TiledError::XmlDecodingError(e) => Some(e as &dyn std::error::Error),
            TiledError::CouldNotOpenFile { err, .. } => Some(err as &dyn std::error::Error),
            _ => None,
        }
    }
}
