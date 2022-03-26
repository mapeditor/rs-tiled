use std::{collections::HashMap, path::Path, sync::Arc};

use xml::attribute::OwnedAttribute;

use crate::{
    animation::{parse_animation, Frame},
    error::TiledError,
    image::Image,
    layers::ObjectLayerData,
    properties::{parse_properties, Properties},
    util::{get_attrs, parse_tag, XmlEventResult},
    ResourceCache, Tileset,
};

pub type TileId = u32;

#[derive(Debug, PartialEq, Clone, Default)]
pub(crate) struct TileData {
    image: Option<Image>,
    properties: Properties,
    collision: Option<ObjectLayerData>,
    animation: Option<Vec<Frame>>,
    tile_type: Option<String>,
    probability: f32,
}

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

    /// Get a reference to the tile's image.
    pub fn image(&self) -> Option<&Image> {
        self.data.image.as_ref()
    }

    /// Get a reference to the tile's properties.
    pub fn properties(&self) -> &Properties {
        &self.data.properties
    }

    /// Get a reference to the tile's collision.
    pub fn collision(&self) -> Option<&ObjectLayerData> {
        self.data.collision.as_ref()
    }

    /// Get a reference to the tile's animation frames.
    pub fn animation(&self) -> Option<&[Frame]> {
        self.data.animation.as_ref().map(Vec::as_slice)
    }

    /// Get a reference to the tile's type.
    pub fn tile_type(&self) -> Option<&str> {
        self.data.tile_type.as_deref()
    }

    /// Get the tile's probability.
    pub fn probability(&self) -> f32 {
        self.data.probability
    }
}

impl TileData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        path_relative_to: &Path,
        for_tileset: Option<Arc<Tileset>>,
        cache: &mut impl ResourceCache,
    ) -> Result<(TileId, TileData), TiledError> {
        let ((tile_type, probability), id) = get_attrs!(
            attrs,
            optionals: [
                ("type", tile_type, |v:String| v.parse().ok()),
                ("probability", probability, |v:String| v.parse().ok()),
            ],
            required: [
                ("id", id, |v:String| v.parse::<u32>().ok()),
            ],
            TiledError::MalformedAttributes("tile must have an id with the correct type".to_string())
        );

        let mut image = Option::None;
        let mut properties = HashMap::new();
        let mut objectgroup = None;
        let mut animation = None;
        parse_tag!(parser, "tile", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, path_relative_to)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "objectgroup" => |attrs| {
                objectgroup = Some(ObjectLayerData::new(parser, attrs, None, for_tileset.as_ref().cloned(), path_relative_to, cache)?.0);
                Ok(())
            },
            "animation" => |_| {
                animation = Some(parse_animation(parser)?);
                Ok(())
            },
        });
        Ok((
            id,
            TileData {
                image,
                properties,
                collision: objectgroup,
                animation,
                tile_type,
                probability: probability.unwrap_or(1.0),
            },
        ))
    }
}
