use std::{fmt, path::PathBuf};

/// Errors which occured when parsing the file
#[derive(Debug)]
#[non_exhaustive]
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
    /// Tried to parse external data of an object without a file location,
    /// e.g. by using Map::parse_reader.
    SourceRequired {
        object_to_parse: String,
    },
    /// The path given is invalid because it isn't contained in any folder.
    PathIsNotFile,
    CouldNotOpenFile {
        path: PathBuf,
        err: std::io::Error,
    },
    /// There was an invalid tile in the map parsed.
    InvalidTileFound,
    /// Unknown encoding or compression format or invalid combination of both (for tile layers)
    InvalidEncodingFormat {
        encoding: Option<String>,
        compression: Option<String>,
    },
    /// There was an error parsing the value of a [`PropertyValue`].
    /// 
    /// [`PropertyValue`]: crate::PropertyValue
    InvalidPropertyValue,
    /// Found an unknown property value type while parsing a [`PropertyValue`].
    /// 
    /// [`PropertyValue`]: crate::PropertyValue
    UnknownPropertyType{name: String},
}

impl fmt::Display for TiledError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            TiledError::MalformedAttributes(s) => write!(fmt, "{}", s),
            TiledError::DecompressingError(e) => write!(fmt, "{}", e),
            TiledError::Base64DecodingError(e) => write!(fmt, "{}", e),
            TiledError::XmlDecodingError(e) => write!(fmt, "{}", e),
            TiledError::PrematureEnd(e) => write!(fmt, "{}", e),
            TiledError::SourceRequired {
                ref object_to_parse,
            } => {
                write!(fmt, "Tried to parse external {} without a file location, e.g. by using Map::parse_reader.", object_to_parse)
            }
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
            TiledError::InvalidEncodingFormat { encoding, compression } => 
                write!(
                    fmt,
                    "Unknown encoding or compression format or invalid combination of both (for tile layers): {} encoding with {} compression",
                    encoding.as_deref().unwrap_or("no"),
                    compression.as_deref().unwrap_or("no")
                ),
            TiledError::InvalidPropertyValue => write!(fmt, "Found invalid property value"),
            TiledError::UnknownPropertyType { name } =>
                write!(fmt, "Found unknown property value type '{}'", name),
        }
    }
}

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
