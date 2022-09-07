use std::{collections::HashMap, path::Path};

use xml::attribute::OwnedAttribute;

use crate::{
    animation::{parse_animation, Frame},
    error::Error,
    image::Image,
    layers::ObjectLayerData,
    parse::xml::properties::parse_properties,
    properties::Properties,
    util::{get_attrs, parse_tag, XmlEventResult},
    ResourceCache, ResourceReader, Result, Tileset,
};

/// A tile ID, local to a tileset.
pub type TileId = u32;

/// Raw data belonging to a tile.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TileData {
    /// The image of the tile. Only set when the tile is part of an "image collection" tileset.
    pub image: Option<Image>,
    /// The custom properties of this tile.
    pub properties: Properties,
    /// The collision shapes of this tile.
    pub collision: Option<ObjectLayerData>,
    /// The animation frames of this tile.
    pub animation: Option<Vec<Frame>>,
    /// The type of this tile.
    pub tile_type: Option<String>,
    /// The probability of this tile.
    pub probability: f32,
}

/// Points to a tile belonging to a tileset.
#[derive(Debug)]
pub struct Tile<'tileset> {
    pub(crate) tileset: &'tileset Tileset,
    pub(crate) data: &'tileset TileData,
}

impl<'tileset> Tile<'tileset> {
    pub(crate) fn new(tileset: &'tileset Tileset, data: &'tileset TileData) -> Self {
        Self { tileset, data }
    }

    /// Get the tileset this tile is from.
    pub fn tileset(&self) -> &'tileset Tileset {
        self.tileset
    }
}

impl<'tileset> std::ops::Deref for Tile<'tileset> {
    type Target = TileData;

    #[inline]
    fn deref(&self) -> &'tileset Self::Target {
        self.data
    }
}
