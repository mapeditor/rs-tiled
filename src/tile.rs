use std::{collections::HashMap, io::Read, path::Path};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    animation::Frame,
    error::TiledError,
    image::Image,
    layers::ObjectLayerData,
    properties::{parse_properties, Properties},
    util::{get_attrs, parse_animation, parse_tag, XmlEventResult},
};

pub type TileId = u32;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Tile {
    pub image: Option<Image>,
    pub properties: Properties,
    pub collision: Option<ObjectLayerData>,
    pub animation: Option<Vec<Frame>>,
    pub tile_type: Option<String>,
    pub probability: f32,
}

impl Tile {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        path_relative_to: &Path,
    ) -> Result<(TileId, Tile), TiledError> {
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
                objectgroup = Some(ObjectLayerData::new(parser, attrs)?.0);
                Ok(())
            },
            "animation" => |_| {
                animation = Some(parse_animation(parser)?);
                Ok(())
            },
        });
        Ok((
            id,
            Tile {
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
