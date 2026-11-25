//! Edit an existing XML map and write it back, retaining tilesets, layer data
//! and TMX features outside the edited elements.
//!
//! ```no_run
//! use tiled::writer::{LayerKind, TmxDocument};
//!
//! let mut map = TmxDocument::open("level.tmx")?;
//! let triggers = map.add_layer(None, LayerKind::Object, "Triggers")?;
//! let id = map.add_rectangle(triggers, None)?;
//! let door = map.object_mut(id).unwrap();
//! door.set_class("door");
//! door.set_attribute("width", 32);
//! door.set_attribute("height", 64);
//! map.save("edited.tmx")?;
//! # Ok::<(), std::io::Error>(())
//! ```
mod properties;
mod xml;

use quick_xml::Writer;
use std::{
    fs,
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};
pub use xml::TmxElement;
use xml::{Node, invalid};

/// The types of layer that can be added to a map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerKind {
    /// A tile layer. Set its contents with [`TmxElement::set_tiles`].
    Tile,
    /// A group of objects.
    Object,
    /// A group of layers.
    Group,
}

/// An editable TMX document. File references in memory are relative to the original map.
#[derive(Clone, Debug)]
pub struct TmxDocument {
    nodes: Vec<Node>,
    root: usize,
    directory: PathBuf,
}

fn directory(path: &Path) -> io::Result<PathBuf> {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
}

impl TmxDocument {
    /// Read a TMX document without loading its external assets.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let nodes = xml::read(BufReader::new(fs::File::open(path)?))?;
        let mut roots = nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| Some((i, n.element()?)));
        let (root, elem) = roots.next().ok_or_else(|| invalid("Missing map element"))?;
        if elem.name != "map" || roots.next().is_some() {
            return Err(invalid("Expected a single TMX map element"));
        }
        Ok(Self {
            nodes,
            root,
            directory: directory(path)?,
        })
    }
    /// Access the map element.
    pub fn map(&self) -> &TmxElement {
        self.nodes[self.root].element().unwrap()
    }
    /// Edit map attributes or properties.
    pub fn map_mut(&mut self) -> &mut TmxElement {
        self.nodes[self.root].element_mut().unwrap()
    }
    /// Iterate tileset references in document order, including embedded tilesets.
    /// Embedded tilesets have no external source.
    pub fn tilesets(&self) -> impl Iterator<Item = (u32, Option<&str>)> {
        self.map()
            .elements()
            .filter(|e| e.name == "tileset")
            .filter_map(|e| {
                Some((
                    e.attribute("firstgid")?.parse().ok()?,
                    e.attribute("source"),
                ))
            })
    }
    /// Return the name of an embedded tileset with this first global tile ID.
    pub fn tileset_name(&self, first_gid: u32) -> Option<&str> {
        self.map()
            .elements()
            .find(|e| {
                e.name == "tileset"
                    && e.attribute("firstgid").and_then(|v| v.parse().ok()) == Some(first_gid)
            })?
            .attribute("name")
    }
    /// Edit a layer anywhere in the map hierarchy.
    pub fn layer_mut(&mut self, id: u32) -> Option<&mut TmxElement> {
        self.map_mut().find_mut(IdKind::Layer, id)
    }
    /// Edit a map object by ID. Tileset collision objects use a separate ID namespace.
    pub fn object_mut(&mut self, id: u32) -> Option<&mut TmxElement> {
        self.map_mut().find_mut(IdKind::Object, id)
    }
    /// Remove a map object by ID. Other objects and their references are left intact.
    /// Returns whether an object was removed.
    pub fn remove_object(&mut self, id: u32) -> bool {
        fn remove(parent: &mut TmxElement, id: u32) -> bool {
            if parent.name == "objectgroup" {
                if let Some(i) = parent.children.iter().position(|node| {
                    node.element()
                        .is_some_and(|e| e.name == "object" && e.id() == Some(id))
                }) {
                    parent.children.remove(i);
                    return true;
                }
                return false;
            }
            parent
                .elements_mut()
                .filter(|e| matches!(e.name.as_str(), "group" | "objectgroup"))
                .any(|layer| remove(layer, id))
        }
        remove(self.map_mut(), id)
    }
    /// Return the ID of the layer containing this map object.
    pub fn object_layer_id(&self, id: u32) -> Option<u32> {
        self.map().find_owner(id)
    }
    /// Add a child to an existing group, or to the root when `parent` is None.
    /// The returned ID is unique among all layers in the map.
    pub fn add_layer(
        &mut self,
        parent: Option<u32>,
        kind: LayerKind,
        name: &str,
    ) -> io::Result<u32> {
        if parent.is_some_and(|id| self.layer_mut(id).is_none_or(|l| l.name != "group")) {
            return Err(invalid("Expected a parent group layer"));
        }
        let id = self.allocate(IdKind::Layer, None)?;
        let mut layer = TmxElement::new(match kind {
            LayerKind::Tile => "layer",
            LayerKind::Object => "objectgroup",
            LayerKind::Group => "group",
        });
        layer.set_attribute("id", id);
        layer.set_attribute("name", name);
        let parent = match parent {
            Some(id) => self.layer_mut(id).unwrap(),
            None => self.map_mut(),
        };
        parent.append(layer);
        Ok(id)
    }
    /// Add a rectangle to an object layer, choosing an unused ID if none is given.
    pub fn add_rectangle(&mut self, layer: u32, id: Option<u32>) -> io::Result<u32> {
        if !self
            .layer_mut(layer)
            .is_some_and(|l| l.name == "objectgroup")
        {
            return Err(invalid("Expected an object layer"));
        }
        let id = self.allocate(IdKind::Object, id)?;
        let mut obj = TmxElement::new("object");
        obj.set_attribute("id", id);
        obj.set_attribute("x", 0);
        obj.set_attribute("y", 0);
        self.layer_mut(layer).unwrap().append(obj);
        Ok(id)
    }
    fn allocate(&mut self, kind: IdKind, requested: Option<u32>) -> io::Result<u32> {
        let counter = match kind {
            IdKind::Object => "nextobjectid",
            IdKind::Layer => "nextlayerid",
        };
        let max = self.map().max_id(kind);
        let next = self
            .map()
            .attribute(counter)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1)
            .max(
                max.checked_add(1)
                    .ok_or_else(|| invalid("TMX ID space exhausted"))?,
            );
        let id = requested.unwrap_or(next);
        if id == 0 || self.map_mut().find_mut(kind, id).is_some() {
            return Err(invalid("TMX ID already in use or zero"));
        }
        self.map_mut().set_attribute(
            counter,
            next.max(
                id.checked_add(1)
                    .ok_or_else(|| invalid("TMX ID space exhausted"))?,
            ),
        );
        Ok(id)
    }
    /// Write a TMX map atomically to `path`. In a different directory, asset and file
    /// property paths are rebased to point to the same files. External assets are never written.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        let dest = directory(path)?;
        let mut temp = tempfile::NamedTempFile::new_in(&dest)?;
        if let Ok(meta) = fs::metadata(path) {
            temp.as_file().set_permissions(meta.permissions())?;
        }
        {
            let mut writer = Writer::new(BufWriter::new(&mut temp));
            for (i, node) in self.nodes.iter().enumerate() {
                if i == self.root && dest != self.directory {
                    let mut root = self.map().clone();
                    root.rebase(&self.directory, &dest)?;
                    root.write(&mut writer)?;
                } else {
                    node.write(&mut writer)?;
                }
            }
            writer.into_inner().flush()?;
        }
        temp.as_file().sync_all()?;
        temp.persist(path)?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IdKind {
    Layer,
    Object,
}

impl TmxElement {
    fn id(&self) -> Option<u32> {
        self.attribute("id")?.parse().ok()
    }
    fn child_id_kind(parent: &str, child: &str) -> Option<IdKind> {
        match (parent, child) {
            ("map" | "group", "layer" | "objectgroup" | "imagelayer" | "group") => {
                Some(IdKind::Layer)
            }
            ("objectgroup", "object") => Some(IdKind::Object),
            _ => None,
        }
    }
    fn find_mut(&mut self, kind: IdKind, id: u32) -> Option<&mut Self> {
        let parent = self.name.as_str();
        for e in self.children.iter_mut().filter_map(Node::element_mut) {
            let Some(ty) = Self::child_id_kind(parent, &e.name) else {
                continue;
            };
            if ty == kind && e.id() == Some(id) {
                return Some(e);
            }
            if let Some(found) = e.find_mut(kind, id) {
                return Some(found);
            }
        }
        None
    }
    fn max_id(&self, kind: IdKind) -> u32 {
        self.elements()
            .filter_map(|e| {
                let ty = Self::child_id_kind(&self.name, &e.name)?;
                Some(
                    e.max_id(kind)
                        .max(if ty == kind { e.id().unwrap_or(0) } else { 0 }),
                )
            })
            .max()
            .unwrap_or(0)
    }
    fn find_owner(&self, id: u32) -> Option<u32> {
        if self.name == "objectgroup" {
            return self
                .elements()
                .any(|e| e.name == "object" && e.id() == Some(id))
                .then(|| self.id())
                .flatten();
        }
        self.elements()
            .filter(|e| Self::child_id_kind(&self.name, &e.name).is_some())
            .find_map(|e| e.find_owner(id))
    }
    /// Update the object's class. Tiled maps may use the `type` or `class` spelling.
    pub fn set_class(&mut self, class: &str) {
        let key = if self.name == "object"
            && (self.attribute("class").is_none() || self.attribute("type").is_some())
        {
            "type"
        } else {
            "class"
        };
        if class.is_empty() {
            self.remove_attribute("class");
            self.remove_attribute("type");
        } else {
            self.set_attribute(key, class);
            if key == "type" {
                self.remove_attribute("class");
            }
        }
    }
    /// Replace finite tile-layer data with row-major global tile IDs (including flip flags).
    /// CSV is emitted with one row per line. Existing chunked data must be edited separately.
    pub fn set_tiles(&mut self, width: u32, height: u32, gids: &[u32]) -> io::Result<()> {
        if self.name != "layer"
            || width == 0
            || height == 0
            || u64::from(width) * u64::from(height) != gids.len() as u64
        {
            return Err(invalid("Tile data dimensions must match the finite layer"));
        }
        if self
            .elements()
            .any(|e| e.name == "data" && e.elements().any(|c| c.name == "chunk"))
        {
            return Err(invalid(
                "Cannot replace chunked layer data with finite cells",
            ));
        }
        self.set_attribute("width", width);
        self.set_attribute("height", height);
        if self.child_mut("data").is_none() {
            self.append(Self::new("data"));
        }
        let data = self.child_mut("data").unwrap();
        data.set_attribute("encoding", "csv");
        data.remove_attribute("compression");
        let mut csv = String::new();
        use std::fmt::Write;
        for (i, v) in gids.iter().enumerate() {
            if i != 0 {
                csv.push(',');
            }
            if i % width as usize == 0 {
                csv.push('\n');
            }
            write!(&mut csv, "{v}").unwrap();
        }
        csv.push('\n');
        data.children = vec![Node::text(&csv)];
        Ok(())
    }
}
