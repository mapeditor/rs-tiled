use std::{fs::File, io::Read, path::Path};

use crate::{DefaultResourceCache, Map, ResourceCache, Result, Tileset};

/// A trait defining types that can load data from a [`ResourcePath`](crate::ResourcePath).
///
/// This trait should be implemented if you wish to load data from a virtual filesystem.
///
/// ## Example
/// TODO: ResourceReader example
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
    fn new() -> Self {
        Self
    }
}

impl ResourceReader for FilesystemResourceReader {
    type Resource = File;
    type Error = std::io::Error;

    fn read_from(&mut self, path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        std::fs::File::open(path)
    }
}

/// A type used for loading [`Map`]s and [`Tileset`]s.
///
/// Internally, it holds a [`ResourceCache`] that, as its name implies, caches intermediate loading
/// artifacts, most notably map tilesets.
///
/// It also contains a [`ResourceReader`] which is the object in charge of providing read handles
/// to files via a [`ResourcePath`](crate::ResourcePath).
///
/// ## Reasoning
/// This type is used for loading operations because they require a [`ResourceCache`] for
/// intermediate artifacts, so using a type for creation can ensure that the cache is reused if
/// loading more than one object is required.
#[derive(Debug, Clone)]
pub struct Loader<
    Cache: ResourceCache = DefaultResourceCache,
    Reader: ResourceReader = FilesystemResourceReader,
> {
    cache: Cache,
    reader: Reader,
}

impl Loader {
    /// Creates a new loader, creating a default resource cache and reader
    /// ([`DefaultResourceCache`] & [`FilesystemResourceReader`] respectively) in the process.
    pub fn new() -> Self {
        Self {
            cache: DefaultResourceCache::new(),
            reader: FilesystemResourceReader::new(),
        }
    }
}

impl<Cache: ResourceCache, Reader: ResourceReader> Loader<Cache, Reader> {
    /// Creates a new loader using a specific resource cache and reader.
    ///
    /// ## Example
    /// ```
    /// # fn main() -> tiled::Result<()> {
    /// use std::{sync::Arc, path::Path};
    ///
    /// use tiled::{Loader, ResourceCache, FilesystemResourceReader};
    ///
    /// /// An example resource cache that doesn't actually cache any resources at all.
    /// struct NoopResourceCache;
    ///
    /// impl ResourceCache for NoopResourceCache {
    ///     fn get_tileset(
    ///         &self,
    ///         _path: impl AsRef<tiled::ResourcePath>,
    ///     ) -> Option<std::sync::Arc<tiled::Tileset>> {
    ///         None
    ///     }
    ///
    ///     fn get_template(
    ///         &self,
    ///         _path: impl AsRef<tiled::ResourcePath>,
    ///     ) -> Option<std::sync::Arc<tiled::Template>> {
    ///         None
    ///     }
    ///
    ///     fn insert_tileset(
    ///         &mut self,
    ///         _path: impl AsRef<tiled::ResourcePath>,
    ///         _tileset: Arc<tiled::Tileset>
    ///     ) {}
    ///
    ///     fn insert_template(
    ///         &mut self,
    ///         _path: impl AsRef<tiled::ResourcePath>,
    ///         _template: Arc<tiled::Template>
    ///     ) {}
    /// }
    ///
    /// let mut loader = Loader::with_cache_and_reader(NoopResourceCache, FilesystemResourceReader);
    ///
    /// let map = loader.load_tmx_map("assets/tiled_base64_external.tmx")?;
    ///
    /// assert_eq!(
    ///     map.tilesets()[0].image.as_ref().unwrap().source,
    ///     Path::new("assets/tilesheet.png")
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_cache_and_reader(cache: Cache, reader: Reader) -> Self {
        Self { cache, reader }
    }

    /// Parses a file hopefully containing a Tiled map and tries to parse it. All external files
    /// will be loaded relative to the path given.
    ///
    /// All intermediate objects such as map tilesets will be stored in the [internal loader cache].
    ///
    /// [internal loader cache]: Loader::cache()
    pub fn load_tmx_map(&mut self, path: impl AsRef<Path>) -> Result<Map> {
        crate::parse::xml::parse_map(path.as_ref(), &mut self.reader, &mut self.cache)
    }

    /// Parses a file hopefully containing a Tiled tileset and tries to parse it. All external files
    /// will be loaded relative to the path given.
    ///
    /// Unless you specifically want to load a tileset, you won't need to call this function. If
    /// you are trying to load a map, simply use [`Loader::load_tmx_map`].
    ///
    /// ## Note
    /// This function will **not** cache the tileset inside the internal [`ResourceCache`], since
    /// in this context it is not an intermediate object.
    pub fn load_tsx_tileset(&mut self, path: impl AsRef<Path>) -> Result<Tileset> {
        crate::parse::xml::parse_tileset(path.as_ref(), &mut self.reader, &mut self.cache)
    }

    /// Returns a reference to the loader's internal [`ResourceCache`].
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Returns a mutable reference to the loader's internal [`ResourceCache`].
    pub fn cache_mut(&mut self) -> &mut Cache {
        &mut self.cache
    }

    /// Returns a reference to the loader's internal [`ResourceReader`].
    pub fn reader(&self) -> &Reader {
        &self.reader
    }

    /// Returns a mutable reference to the loader's internal [`ResourceReader`].
    pub fn reader_mut(&mut self) -> &mut Reader {
        &mut self.reader
    }

    /// Consumes the loader and returns its internal [`ResourceCache`] and [`ResourceReader`].
    pub fn into_inner(self) -> (Cache, Reader) {
        (self.cache, self.reader)
    }
}
