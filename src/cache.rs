use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::Tileset;

pub type ResourcePath = Path;
pub type ResourcePathBuf = PathBuf;

pub trait ResourceCache {
    fn get_tileset(&self, path: impl AsRef<ResourcePath>) -> Option<Arc<Tileset>>;
    fn insert_tileset(&mut self, path: PathBuf, tileset: Arc<Tileset>);
}

/// A cache that identifies resources by their path in the user's filesystem.
pub struct FilesystemResourceCache {
    tilesets: HashMap<ResourcePathBuf, Arc<Tileset>>,
}

impl FilesystemResourceCache {
    pub fn new() -> Self {
        Self {
            tilesets: HashMap::new(),
        }
    }
}

impl ResourceCache for FilesystemResourceCache {
    fn get_tileset(&self, path: impl AsRef<ResourcePath>) -> Option<Arc<Tileset>> {
        self.tilesets.get(path.as_ref()).map(Clone::clone)
    }

    fn insert_tileset(&mut self, path: PathBuf, tileset: Arc<Tileset>) {
        self.tilesets.insert(path, tileset);
    }
}
