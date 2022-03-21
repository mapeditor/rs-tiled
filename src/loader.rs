use std::{fs::File, io::Read, path::Path};

use crate::{FilesystemResourceCache, Map, ResourceCache, Result, Tileset};

/// A trait defining types that can load data from a [`ResourcePath`](crate::ResourcePath).
/// 
/// This trait should be implemented if you wish to load data from a virtual filesystem.
/// 
/// ## Example
/// TODO
pub trait ResourceReader {
    /// The type of the resource that the reader provides. For example, for
    /// [`FilesystemResourceReader`], this is defined as [`File`].
    type Resource: Read;
    /// The type that is returned if [`read_from()`](Self::read_from()) fails. For example, for
    /// [`FilesystemResourceReader`], this is defined as [`std::io::Error`].
    type Error: std::error::Error + 'static;

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
/// ## Reasoning
/// This type is used for loading operations because they require a [`ResourceCache`] for
/// intermediate artifacts, so using a type for creation can ensure that the cache is reused if
/// loading more than one object is required.
#[derive(Debug, Clone)]
pub struct Loader<
    Cache: ResourceCache = FilesystemResourceCache,
    Reader: ResourceReader = FilesystemResourceReader,
> {
    cache: Cache,
    reader: Reader,
}

impl Loader<FilesystemResourceCache> {
    /// Creates a new loader, creating a default ([`FilesystemResourceCache`]) resource cache in the process.
    pub fn new() -> Self {
        Self {
            cache: FilesystemResourceCache::new(),
            reader: FilesystemResourceReader::new(),
        }
    }
}

impl<Cache: ResourceCache, Reader: ResourceReader> Loader<Cache, Reader> {
    /// Creates a new loader using a specific resource cache.
    ///
    /// ## Example
    /// ```
    /// # fn main() -> tiled::Result<()> {
    /// use std::{sync::Arc, path::Path};
    ///
    /// use tiled::{Loader, ResourceCache};
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
    ///     fn get_or_try_insert_tileset_with<F, E>(
    ///         &mut self,
    ///         _path: tiled::ResourcePathBuf,
    ///         f: F,
    ///     ) -> Result<std::sync::Arc<tiled::Tileset>, E>
    ///     where
    ///         F: FnOnce() -> Result<tiled::Tileset, E>,
    ///     {
    ///         f().map(Arc::new)
    ///     }
    /// }
    ///
    /// let mut loader = Loader::with_cache(NoopResourceCache);
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
    /// you are trying to load a map, simply use [`Loader::load_tmx_map`] or
    /// [`Loader::load_tmx_map_from`].
    ///
    /// If you need to parse a reader object instead, use [Loader::load_tsx_tileset_from()].
    pub fn load_tsx_tileset(&mut self, path: impl AsRef<Path>) -> Result<Tileset> {
        crate::parse::xml::parse_tileset(path.as_ref(), &mut self.reader)
    }

    /// Returns a reference to the loader's internal [`ResourceCache`].
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Returns a reference to the loader's internal [`ResourceReader`].
    pub fn reader(&self) -> &Reader {
        &self.reader
    }

    /// Consumes the loader and returns its internal [`ResourceCache`] and [`ResourceReader`].
    pub fn into_inner(self) -> (Cache, Reader) {
        (self.cache, self.reader)
    }
}
