use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{Template, Tileset};

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
    /// use tiled::{Tileset, Loader, ResourceCache};
    /// # use tiled::Result;
    /// # use std::sync::Arc;
    ///
    /// # fn main() -> Result<()> {
    /// let mut loader = Loader::new();
    /// let path = "assets/tilesheet.tsx";
    ///
    /// assert!(loader.cache().get_tileset(path).is_none());
    /// let tileset = Arc::new(loader.load_tsx_tileset(path)?);
    /// loader.cache_mut().insert_tileset(path, tileset);
    /// assert!(loader.cache().get_tileset(path).is_some());
    /// # Ok(())
    /// # }
    /// ```
    fn get_tileset(&self, path: impl AsRef<ResourcePath>) -> Option<Arc<Tileset>>;
    /// Insert a new tileset into the cache.
    ///
    /// See [`Self::get_tileset()`] for an example.
    fn insert_tileset(&mut self, path: impl AsRef<ResourcePath>, tileset: Arc<Tileset>);
    /// Obtains a template from the cache, if it exists.
    fn get_template(&self, path: impl AsRef<ResourcePath>) -> Option<Arc<Template>>;
    /// Insert a new template into the cache.
    fn insert_template(&mut self, path: impl AsRef<ResourcePath>, tileset: Arc<Template>);
}

/// A cache that identifies resources by their path, storing them in a [`HashMap`].
#[derive(Debug, Default)]
pub struct DefaultResourceCache {
    /// The tilesets cached until now.
    pub tilesets: HashMap<ResourcePathBuf, Arc<Tileset>>,
    /// The templates cached until now.
    pub templates: HashMap<ResourcePathBuf, Arc<Template>>,
}

impl DefaultResourceCache {
    /// Creates an empty [`DefaultResourceCache`].
    pub fn new() -> Self {
        Self {
            tilesets: HashMap::new(),
            templates: HashMap::new(),
        }
    }
}

impl ResourceCache for DefaultResourceCache {
    fn get_tileset(&self, path: impl AsRef<ResourcePath>) -> Option<Arc<Tileset>> {
        self.tilesets.get(path.as_ref()).map(Clone::clone)
    }

    fn insert_tileset(&mut self, path: impl AsRef<ResourcePath>, tileset: Arc<Tileset>) {
        self.tilesets.insert(path.as_ref().to_path_buf(), tileset);
    }

    fn get_template(&self, path: impl AsRef<ResourcePath>) -> Option<Arc<Template>> {
        self.templates.get(path.as_ref()).map(Clone::clone)
    }

    fn insert_template(&mut self, path: impl AsRef<ResourcePath>, tileset: Arc<Template>) {
        self.templates.insert(path.as_ref().to_path_buf(), tileset);
    }
}
