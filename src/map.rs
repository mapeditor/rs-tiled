//! Structures related to Tiled maps.

use std::{collections::HashMap, fmt, path::Path, str::FromStr, sync::Arc};

use xml::attribute::OwnedAttribute;

use crate::{
    error::{Error, Result},
    layers::{LayerData, LayerTag},
    parse::common::tileset::EmbeddedParseResultType,
    properties::{Color, Properties},
    tileset::Tileset,
    util::{get_attrs, parse_tag, XmlEventResult},
    Layer, ResourceCache, ResourceReader,
};

pub(crate) struct MapTilesetGid {
    pub first_gid: Gid,
    pub tileset: Arc<Tileset>,
}

/// All Tiled map files will be parsed into this. Holds all the layers and tilesets.
#[derive(PartialEq, Clone, Debug)]
pub struct Map {
    pub(crate) version: String,
    /// The way tiles are laid out in the map.
    pub orientation: Orientation,
    /// Width of the map, in tiles.
    ///
    /// ## Note
    /// There is no guarantee that this value will be the same as the width from its tile layers.
    pub width: u32,
    /// Height of the map, in tiles.
    ///
    /// ## Note
    /// There is no guarantee that this value will be the same as the height from its tile layers.
    pub height: u32,
    /// Tile width, in pixels.
    ///
    /// ## Note
    /// This value along with [`Self::tile_height`] determine the general size of the map, and
    /// individual tiles may have different sizes. As such, there is no guarantee that this value
    /// will be the same as the one from the tilesets the map is using.
    pub tile_width: u32,
    /// Tile height, in pixels.
    ///
    /// ## Note
    /// This value along with [`Self::tile_width`] determine the general size of the map, and
    /// individual tiles may have different sizes. As such, there is no guarantee that this value
    /// will be the same as the one from the tilesets the map is using.
    pub tile_height: u32,
    /// The tilesets present on this map.
    pub(crate) tilesets: Vec<Arc<Tileset>>,
    /// The layers present in this map.
    pub(crate) layers: Vec<LayerData>,
    /// The custom properties of this map.
    pub properties: Properties,
    /// The background color of this map, if any.
    pub background_color: Option<Color>,
    pub(crate) infinite: bool,
}

impl Map {
    /// The TMX format version this map was saved to. Equivalent to the map file's `version`
    /// attribute.
    pub fn version(&self) -> &str {
        self.version.as_ref()
    }

    /// Whether this map is infinite. An infinite map has no fixed size and can grow in all
    /// directions. Its layer data is stored in chunks. This value determines whether the map's
    /// tile layers are [`FiniteTileLayer`](crate::FiniteTileLayer)s or [`crate::InfiniteTileLayer`](crate::InfiniteTileLayer)s.
    pub fn infinite(&self) -> bool {
        self.infinite
    }
}

impl Map {
    /// Get a reference to the map's tilesets.
    #[inline]
    pub fn tilesets(&self) -> &[Arc<Tileset>] {
        self.tilesets.as_ref()
    }

    /// Get an iterator over all the layers in the map in ascending order of their layer index.
    ///
    /// ## Example
    /// ```
    /// # use tiled::Loader;
    /// #
    /// # fn main() {
    /// # struct Renderer;
    /// # impl Renderer {
    /// #     fn render(&self, _: tiled::TileLayer) {}
    /// # }
    /// # let my_renderer = Renderer;
    /// # let map = Loader::new()
    /// #     .load_tmx_map("assets/tiled_group_layers.tmx")
    /// #     .unwrap();
    /// #
    /// let tile_layers = map.layers().filter_map(|layer| match layer.layer_type() {
    ///     tiled::LayerType::Tiles(layer) => Some(layer),
    ///     _ => None,
    /// });
    ///
    /// for layer in tile_layers {
    ///     my_renderer.render(layer);
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn layers(&self) -> impl ExactSizeIterator<Item = Layer> {
        self.layers.iter().map(move |layer| Layer::new(self, layer))
    }

    /// Returns the layer that has the specified index, if it exists.
    pub fn get_layer(&self, index: usize) -> Option<Layer> {
        self.layers.get(index).map(|data| Layer::new(self, data))
    }
}

/// Represents the way tiles are laid out in a map.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(missing_docs)]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

impl FromStr for Orientation {
    // TODO(0.11): Change error type to OrientationParseErr or similar
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "orthogonal" => Ok(Orientation::Orthogonal),
            "isometric" => Ok(Orientation::Isometric),
            "staggered" => Ok(Orientation::Staggered),
            "hexagonal" => Ok(Orientation::Hexagonal),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Orientation::Orthogonal => write!(f, "orthogonal"),
            Orientation::Isometric => write!(f, "isometric"),
            Orientation::Staggered => write!(f, "staggered"),
            Orientation::Hexagonal => write!(f, "hexagonal"),
        }
    }
}

/// A Tiled global tile ID.
///
/// These are used to identify tiles in a map. Since the map may have more than one tileset, an
/// unique mapping is required to convert the tiles' local tileset ID to one which will work nicely
/// even if there is more than one tileset.
///
/// Tiled also treats GID 0 as empty space, which means that the first tileset in the map will have
/// a starting GID of 1.
///
/// See also: https://doc.mapeditor.org/en/latest/reference/global-tile-ids/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Gid(pub u32);

impl Gid {
    /// The GID representing an empty tile in the map.
    #[allow(dead_code)]
    pub const EMPTY: Gid = Gid(0);
}
