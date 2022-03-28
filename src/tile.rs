use std::{collections::HashMap, path::Path};

use xml::attribute::OwnedAttribute;

use crate::{
    animation::{parse_animation, Frame},
    error::Error,
    image::Image,
    layers::ObjectLayerData,
    properties::{parse_properties, Properties},
    util::{get_attrs, parse_tag, XmlEventResult},
    Result, Tileset,
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

impl TileData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        path_relative_to: &Path,
    ) -> Result<(TileId, TileData)> {
        let ((tile_type, probability), id) = get_attrs!(
            attrs,
            optionals: [
                ("type", tile_type, |v:String| v.parse().ok()),
                ("probability", probability, |v:String| v.parse().ok()),
            ],
            required: [
                ("id", id, |v:String| v.parse::<u32>().ok()),
            ],
            Error::MalformedAttributes("tile must have an id with the correct type".to_string())
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
                objectgroup = Some(ObjectLayerData::new(parser, attrs, None)?.0);
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
