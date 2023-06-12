use std::{fs::File, io::Read, path::Path};

/// A trait defining types that can load data from a [`ResourcePath`](crate::ResourcePath).
///
/// This trait should be implemented if you wish to load data from a virtual filesystem.
///
/// ## Example
/// ```
/// use std::io::Cursor;
///
/// /// Basic example reader impl that just keeps a few resources in memory
/// struct MemoryReader;
///
/// impl tiled::ResourceReader for MemoryReader {
///     type Resource = Cursor<&'static [u8]>;
///     type Error = std::io::Error;
///
///     fn read_from(&mut self, path: &std::path::Path) -> std::result::Result<Self::Resource, Self::Error> {
///         if path == std::path::Path::new("my_map.tmx") {
///             Ok(Cursor::new(include_bytes!("../assets/tiled_xml.tmx")))
///         } else {
///             Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
///         }
///     }
/// }
/// ```
pub trait ResourceReader {
    /// The type of the resource that the reader provides. For example, for
    /// [`FilesystemResourceReader`], this is defined as [`File`].
    type Resource: Read;
    /// The type that is returned if [`read_from()`](Self::read_from()) fails. For example, for
    /// [`FilesystemResourceReader`], this is defined as [`std::io::Error`].
    type Error: std::error::Error + Send + Sync + 'static;

    /// Try to return a reader object from a path into the resources filesystem.
    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error>;
}

/// A [`ResourceReader`] that reads from [`File`] handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilesystemResourceReader;

impl FilesystemResourceReader {
    /// Creates a new [`FilesystemResourceReader`].
    pub fn new() -> Self {
        Self
    }
}

impl ResourceReader for FilesystemResourceReader {
    type Resource = File;
    type Error = std::io::Error;

    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        File::open(path)
    }
}

impl<T, R, E> ResourceReader for T
where
    T: for<'a> Fn(&'a Path) -> Result<R, E>,
    R: Read,
    E: std::error::Error + Send + Sync + 'static,
{
    type Resource = R;

    type Error = E;

    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        self(path)
    }
}
