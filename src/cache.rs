use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::Tileset;

pub trait ResourceCache {
    fn get_tileset(&self, path: &Path) -> Option<&Tileset>;
    fn get_or_insert_tileset(&mut self, path: &Path, tileset: Tileset) -> &Tileset;
}

pub struct DefaultResourceCache {
    tilesets: HashMap<PathBuf, Tileset>,
}

impl DefaultResourceCache {
    pub fn new() -> Self {
        Self {
            tilesets: HashMap::new(),
        }
    }
}

impl ResourceCache for DefaultResourceCache {
    fn get_tileset(&self, path: &Path) -> Option<&Tileset> {
        self.tilesets.get(path)
    }

    fn get_or_insert_tileset(&mut self, path: &Path, tileset: Tileset) -> &Tileset {
        self.tilesets.entry(path.to_owned()).or_insert(tileset)
    }
}
