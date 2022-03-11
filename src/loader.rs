use std::{fs::File, io::Read, path::Path};

use crate::{Error, FilesystemResourceCache, Map, ResourceCache, Result, Tileset};

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
pub struct Loader<Cache: ResourceCache = FilesystemResourceCache> {
    cache: Cache,
}

impl Loader<FilesystemResourceCache> {
    /// Creates a new loader, creating a default ([`FilesystemResourceCache`]) resource cache in the process.
    pub fn new() -> Self {
        Self {
            cache: FilesystemResourceCache::new(),
        }
    }
}

impl<Cache: ResourceCache> Loader<Cache> {
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
    pub fn with_cache(cache: Cache) -> Self {
        Self { cache }
    }

    /// Parses a file hopefully containing a Tiled map and tries to parse it. All external files
    /// will be loaded relative to the path given.
    ///
    /// All intermediate objects such as map tilesets will be stored in the [internal loader cache].
    ///
    /// If you need to parse a reader object instead, use [Loader::load_tmx_map_from()].
    ///
    /// [internal loader cache]: Loader::cache()
    pub fn load_tmx_map(&mut self, path: impl AsRef<Path>) -> Result<Map> {
        let reader = File::open(path.as_ref()).map_err(|err| Error::CouldNotOpenFile {
            path: path.as_ref().to_owned(),
            err,
        })?;
        crate::parse::xml::parse_map(reader, path.as_ref(), &mut self.cache)
    }

    /// Parses a map out of a reader hopefully containing the contents of a Tiled file.
    ///
    /// This augments [`load_tmx_map`] with a custom reader: some engines (e.g. Amethyst) simply
    /// hand over a byte stream and file location for parsing, in which case this function may be
    /// required.
    ///
    /// If you need to parse a file in the filesystem instead, [`load_tmx_map`] might be
    /// more convenient.
    ///
    /// The path is used for external dependencies such as tilesets or images. It is required.
    /// If the map if fully embedded and doesn't refer to external files, you may input an arbitrary
    /// path; the library won't read from the filesystem if it is not required to do so.
    ///
    /// All intermediate objects such as map tilesets will be stored in the [internal loader cache].
    ///
    /// [internal loader cache]: Loader::cache()
    /// [`load_tmx_map`]: Loader::load_tmx_map()
    pub fn load_tmx_map_from(&mut self, reader: impl Read, path: impl AsRef<Path>) -> Result<Map> {
        crate::parse::xml::parse_map(reader, path.as_ref(), &mut self.cache)
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
        let reader = File::open(path.as_ref()).map_err(|err| Error::CouldNotOpenFile {
            path: path.as_ref().to_owned(),
            err,
        })?;
        crate::parse::xml::parse_tileset(reader, path.as_ref())
    }

    /// Parses a tileset out of a reader hopefully containing the contents of a Tiled tileset.
    /// Uses the `path` parameter as the root for any relative paths found in the tileset.
    ///
    /// Unless you specifically want to load a tileset, you won't need to call this function. If
    /// you are trying to load a map, simply use [`Loader::load_tmx_map`] or
    /// [`Loader::load_tmx_map_from`].
    ///
    /// ## Example
    /// ```
    /// use std::fs::File;
    /// use std::path::PathBuf;
    /// use std::io::BufReader;
    /// use tiled::Loader;
    ///
    /// let path = "assets/tilesheet.tsx";
    /// // Note: This is just an example, if you actually need to load a file use `load_tsx_tileset`
    /// // instead.
    /// let reader = BufReader::new(File::open(path).unwrap());
    /// let mut loader = Loader::new();
    /// let tileset = loader.load_tsx_tileset_from(reader, path).unwrap();
    ///
    /// assert_eq!(tileset.image.unwrap().source, PathBuf::from("assets/tilesheet.png"));
    /// ```
    pub fn load_tsx_tileset_from(
        &self,
        reader: impl Read,
        path: impl AsRef<Path>,
    ) -> Result<Tileset> {
        // This function doesn't need the cache right now, but will do once template support is in
        crate::parse::xml::parse_tileset(reader, path.as_ref())
    }

    /// Returns a reference to the loader's internal [`ResourceCache`].
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Consumes the loader and returns its internal [`ResourceCache`].
    pub fn into_cache(self) -> Cache {
        self.cache
    }
}
