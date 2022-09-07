use std::collections::HashMap;
use std::path::{Path, PathBuf};

use xml::attribute::OwnedAttribute;

use crate::error::{Error, Result};
use crate::image::Image;
use crate::properties::Properties;
use crate::tile::TileData;
use crate::{util::*, Gid, ResourceCache, ResourceReader, Tile, TileId};

mod wangset;
pub use wangset::{WangColor, WangId, WangSet, WangTile};

/// A collection of tiles for usage in maps and template objects.
///
/// Also see the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tileset).
#[derive(Debug, PartialEq, Clone)]
pub struct Tileset {
    /// The name of the tileset, set by the user.
    pub name: String,
    /// The (maximum) width in pixels of the tiles in this tileset. Irrelevant for [image collection]
    /// tilesets.
    ///
    /// [image collection]: Self::image
    pub tile_width: u32,
    /// The (maximum) height in pixels of the tiles in this tileset. Irrelevant for [image collection]
    /// tilesets.
    ///
    /// [image collection]: Self::image
    pub tile_height: u32,
    /// The spacing in pixels between the tiles in this tileset (applies to the tileset image).
    /// Irrelevant for image collection tilesets.
    pub spacing: u32,
    /// The margin around the tiles in this tileset (applies to the tileset image).
    /// Irrelevant for image collection tilesets.
    pub margin: u32,
    /// The number of tiles in this tileset. Note that tile IDs don't always have a connection with
    /// the tile count, and as such there may be tiles with an ID bigger than the tile count.
    pub tilecount: u32,
    /// The number of tile columns in the tileset. Editable for image collection tilesets, otherwise
    /// calculated using [image](Self::image) width, [tile width](Self::tile_width),
    /// [spacing](Self::spacing) and [margin](Self::margin).
    pub columns: u32,

    /// A tileset can either:
    /// * have a single spritesheet `image` in `tileset` ("regular" tileset);
    /// * have zero images in `tileset` and one `image` per `tile` ("image collection" tileset).
    ///
    /// --------
    /// - Source: [tiled issue #2117](https://github.com/mapeditor/tiled/issues/2117)
    /// - Source: [`columns` documentation](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tileset)
    pub image: Option<Image>,

    /// All the tiles present in this tileset, indexed by their local IDs.
    pub(crate) tiles: HashMap<TileId, TileData>,

    /// All the wangsets present in this tileset.
    pub wang_sets: Vec<WangSet>,

    /// The custom properties of the tileset.
    pub properties: Properties,
}

impl Tileset {
    /// Gets the tile with the specified ID from the tileset.
    #[inline]
    pub fn get_tile(&self, id: TileId) -> Option<Tile> {
        self.tiles.get(&id).map(|data| Tile::new(self, data))
    }

    /// Iterates through the tiles from this tileset.
    #[inline]
    pub fn tiles(&self) -> impl ExactSizeIterator<Item = (TileId, Tile)> {
        self.tiles
            .iter()
            .map(move |(id, data)| (*id, Tile::new(self, data)))
    }
}
