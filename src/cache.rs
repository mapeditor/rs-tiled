use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::Tileset;

pub type ResourcePath = Path;
pub type ResourcePathBuf = PathBuf;

pub trait ResourceCache {
    fn get_tileset(&self, path: impl AsRef<ResourcePath>) -> Option<Rc<Tileset>>;
    fn get_or_insert_tileset(&mut self, path: ResourcePathBuf, tileset: Tileset) -> Rc<Tileset>;
    fn get_or_insert_tileset_with<F>(&mut self, path: ResourcePathBuf, f: F) -> Rc<Tileset>
    where
        F: FnOnce() -> Tileset;
    fn get_or_try_insert_tileset_with<F, E>(
        &mut self,
        path: ResourcePathBuf,
        f: F,
    ) -> Result<Rc<Tileset>, E>
    where
        F: FnOnce() -> Result<Tileset, E>;
}

pub struct DefaultResourceCache {
    tilesets: HashMap<ResourcePathBuf, Rc<Tileset>>,
}

impl DefaultResourceCache {
    pub fn new() -> Self {
        Self {
            tilesets: HashMap::new(),
        }
    }
}

impl ResourceCache for DefaultResourceCache {
    fn get_tileset(&self, path: impl AsRef<ResourcePath>) -> Option<Rc<Tileset>> {
        self.tilesets.get(path.as_ref()).map(Clone::clone)
    }

    fn get_or_insert_tileset(&mut self, path: ResourcePathBuf, tileset: Tileset) -> Rc<Tileset> {
        self.tilesets
            .entry(path)
            .or_insert(Rc::new(tileset))
            .clone()
    }

    fn get_or_insert_tileset_with<F>(&mut self, path: ResourcePathBuf, f: F) -> Rc<Tileset>
    where
        F: FnOnce() -> Tileset,
    {
        self.tilesets
            .entry(path)
            .or_insert_with(|| Rc::new(f()))
            .clone()
    }

    fn get_or_try_insert_tileset_with<F, E>(
        &mut self,
        path: ResourcePathBuf,
        f: F,
    ) -> Result<Rc<Tileset>, E>
    where
        F: FnOnce() -> Result<Tileset, E>,
    {
        Ok(match self.tilesets.entry(path) {
            std::collections::hash_map::Entry::Occupied(o) => o.into_mut(),
            std::collections::hash_map::Entry::Vacant(v) => v.insert(Rc::new(f()?)),
        }
        .clone())
    }
}
