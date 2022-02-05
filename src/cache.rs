use std::{collections::HashMap, path::PathBuf};

use crate::Tileset;

pub type ResourcePath = PathBuf;

pub trait ResourceCache {
    fn get_tileset(&self, path: &ResourcePath) -> Option<&Tileset>;
    fn get_or_insert_tileset(&mut self, path: ResourcePath, tileset: Tileset) -> &Tileset;
    fn get_or_insert_tileset_with<F>(&mut self, path: ResourcePath, f: F) -> &Tileset
    where
        F: FnOnce() -> Tileset;
    fn get_or_try_insert_tileset_with<F, E>(
        &mut self,
        path: ResourcePath,
        f: F,
    ) -> Result<&Tileset, E>
    where
        F: FnOnce() -> Result<Tileset, E>;
}

pub struct DefaultResourceCache {
    tilesets: HashMap<ResourcePath, Tileset>,
}

impl DefaultResourceCache {
    pub fn new() -> Self {
        Self {
            tilesets: HashMap::new(),
        }
    }
}

impl ResourceCache for DefaultResourceCache {
    fn get_tileset(&self, path: &ResourcePath) -> Option<&Tileset> {
        self.tilesets.get(path)
    }

    fn get_or_insert_tileset(&mut self, path: ResourcePath, tileset: Tileset) -> &Tileset {
        self.tilesets.entry(path.to_owned()).or_insert(tileset)
    }

    fn get_or_insert_tileset_with<F>(&mut self, path: ResourcePath, f: F) -> &Tileset
    where
        F: FnOnce() -> Tileset,
    {
        self.tilesets.entry(path.to_owned()).or_insert_with(f)
    }

    fn get_or_try_insert_tileset_with<F, E>(
        &mut self,
        path: ResourcePath,
        f: F,
    ) -> Result<&Tileset, E>
    where
        F: FnOnce() -> Result<Tileset, E>,
    {
        if !self.tilesets.contains_key(&path) {
            Ok(self.tilesets.entry(path).or_insert(f()?))
        } else {
            Ok(self.tilesets.get(&path).unwrap())
        }
    }
}
