use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::Tileset;

/// A reference type that is used to refer to a resource. For the owned variant, see [`ResourcePathBuf`].
pub type ResourcePath = Path;
/// An owned type that is used to refer to a resource. For the non-owned variant, see [`ResourcePath`].
pub type ResourcePathBuf = PathBuf;

/// A trait identifying a data type that holds resources (such as tilesets) and maps them to a
/// [`ResourcePath`] to prevent loading them more than once. Normally you don't need to use this
/// type yourself unless you want to create a custom caching solution to, for instance, integrate
/// with your own.
///
/// If you simply want to load a map or tileset, use the [`Loader`](crate::Loader) type.
pub trait ResourceCache {
    /// Obtains a tileset from the cache, if it exists.
    ///
    /// # Example
    /// ```
    /// use std::fs::File;
    /// use tiled::{FilesystemResourceReader, Tileset, Loader, ResourceCache};
    /// # use tiled::Result;
    /// # fn main() -> Result<()> {
    /// let mut loader = Loader::new();
    /// let path = "assets/tilesheet.tsx";
    ///
    /// assert!(loader.cache().get_tileset(path).is_none());
    /// loader.load_tmx_map("assets/tiled_base64_external.tmx");
    /// assert!(loader.cache().get_tileset(path).is_some());
    /// # Ok(())
    /// # }
    /// ```
    fn get_tileset(&self, path: impl AsRef<ResourcePath>) -> Option<Arc<Tileset>>;

    /// Returns the tileset mapped to `path` if it exists, otherwise calls `f` and, depending on its
    /// result, it will:
    /// - Insert the object into the cache, if the result was [`Ok`].
    /// - Return the error and leave the cache intact, if the result was [`Err`].
    ///
    /// ## Note
    /// This function is normally only used internally; there are not many instances where it is
    /// callable outside of the library implementation, since the cache is normally owned by the
    /// loader anyways.
    fn get_or_try_insert_tileset_with<F, E>(
        &mut self,
        path: ResourcePathBuf,
        f: F,
    ) -> Result<Arc<Tileset>, E>
    where
        F: FnOnce() -> Result<Tileset, E>;
}

/// A cache that identifies resources by their path, storing a map of them.
#[derive(Debug, Default)]
pub struct DefaultResourceCache {
    tilesets: HashMap<ResourcePathBuf, Arc<Tileset>>,
}

impl DefaultResourceCache {
    /// Creates an empty [`DefaultResourceCache`].
    pub fn new() -> Self {
        Self {
            tilesets: HashMap::new(),
        }
    }
}

impl ResourceCache for DefaultResourceCache {
    fn get_tileset(&self, path: impl AsRef<ResourcePath>) -> Option<Arc<Tileset>> {
        self.tilesets.get(path.as_ref()).map(Clone::clone)
    }

    fn get_or_try_insert_tileset_with<F, E>(
        &mut self,
        path: ResourcePathBuf,
        f: F,
    ) -> Result<Arc<Tileset>, E>
    where
        F: FnOnce() -> Result<Tileset, E>,
    {
        Ok(match self.tilesets.entry(path) {
            std::collections::hash_map::Entry::Occupied(o) => o.into_mut(),
            std::collections::hash_map::Entry::Vacant(v) => v.insert(Arc::new(f()?)),
        }
        .clone())
    }
}
