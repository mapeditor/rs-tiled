use std::{fs::File, io::Read, path::Path};

use crate::{Error, FilesystemResourceCache, Map, ResourceCache, Result};

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
    /// Creates a new loader, creating a default resource cache in the process.
    pub fn new() -> Self {
        Self {
            cache: FilesystemResourceCache::new(),
        }
    }
}

impl<Cache: ResourceCache> Loader<Cache> {
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

    /// Parses a reader hopefully containing the contents of a Tiled file and tries to
    /// parse it. This augments [`load_tmx_map`](Loader::load_tmx_map()) with a custom reader: some
    /// engines (e.g. Amethyst) simply hand over a byte stream and file location for parsing,
    /// in which case this function may be required.
    ///
    /// If you need to parse a file in the filesystem instead, [Loader::load_tmx_map()] is more
    /// convenient.
    ///
    /// The path is used for external dependencies such as tilesets or images. It is required.
    /// If the map if fully embedded and doesn't refer to external files, you may input an arbitrary
    /// path; the library won't read from the filesystem if it is not required to do so.
    ///
    /// All intermediate objects such as map tilesets will be stored in the [internal loader cache].
    ///
    /// [internal loader cache]: Loader::cache()
    pub fn load_tmx_map_from(&mut self, reader: impl Read, path: impl AsRef<Path>) -> Result<Map> {
        crate::parse::xml::parse_map(reader, path.as_ref(), &mut self.cache)
    }

    pub fn cache(&self) -> &Cache {
        &self.cache
    }
}
