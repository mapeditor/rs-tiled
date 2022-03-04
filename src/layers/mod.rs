use std::path::Path;

use xml::attribute::OwnedAttribute;

use crate::{error::TiledError, properties::Properties, util::*, Color, Map, MapTilesetGid};

mod image;
pub use image::*;
mod object;
pub use object::*;
mod tile;
pub use tile::*;
mod group;
pub use group::*;

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum LayerDataType {
    TileLayer(TileLayerData),
    ObjectLayer(ObjectLayerData),
    ImageLayer(ImageLayerData),
    GroupLayer(GroupLayerData),
}

#[derive(Clone, Copy)]
pub(crate) enum LayerTag {
    TileLayer,
    ObjectLayer,
    ImageLayer,
    GroupLayer,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LayerData {
    name: String,
    id: u32,
    visible: bool,
    offset_x: f32,
    offset_y: f32,
    parallax_x: f32,
    parallax_y: f32,
    opacity: f32,
    tint_color: Option<Color>,
    properties: Properties,
    layer_type: LayerDataType,
}

impl LayerData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tag: LayerTag,
        infinite: bool,
        map_path: &Path,
        tilesets: &[MapTilesetGid],
    ) -> Result<Self, TiledError> {
        let (
            (opacity, tint_color, visible, offset_x, offset_y, parallax_x, parallax_y, name, id),
            (),
        ) = get_attrs!(
            attrs,
            optionals: [
                ("opacity", opacity, |v:String| v.parse().ok()),
                ("tintcolor", tint_color, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("offsetx", offset_x, |v:String| v.parse().ok()),
                ("offsety", offset_y, |v:String| v.parse().ok()),
                ("parallaxx", parallax_x, |v:String| v.parse().ok()),
                ("parallaxy", parallax_y, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
                ("id", id, |v:String| v.parse().ok()),
            ],
            required: [
            ],

            TiledError::MalformedAttributes("layer parsing error, no id attribute found".to_string())
        );

        let (ty, properties) = match tag {
            LayerTag::TileLayer => {
                let (ty, properties) = TileLayerData::new(parser, attrs, infinite, tilesets)?;
                (LayerDataType::TileLayer(ty), properties)
            }
            LayerTag::ObjectLayer => {
                let (ty, properties) = ObjectLayerData::new(parser, attrs, Some(tilesets))?;
                (LayerDataType::ObjectLayer(ty), properties)
            }
            LayerTag::ImageLayer => {
                let (ty, properties) = ImageLayerData::new(parser, map_path)?;
                (LayerDataType::ImageLayer(ty), properties)
            }
            LayerTag::GroupLayer => {
                let (ty, properties) = GroupLayerData::new(parser, infinite, map_path, tilesets)?;
                (LayerDataType::GroupLayer(ty), properties)
            }
        };

        Ok(Self {
            visible: visible.unwrap_or(true),
            offset_x: offset_x.unwrap_or(0.0),
            offset_y: offset_y.unwrap_or(0.0),
            parallax_x: parallax_x.unwrap_or(1.0),
            parallax_y: parallax_y.unwrap_or(1.0),
            opacity: opacity.unwrap_or(1.0),
            tint_color,
            name: name.unwrap_or_default(),
            id: id.unwrap_or(0),
            properties,
            layer_type: ty,
        })
    }
}

map_wrapper!(
    #[doc = "A generic map layer, accessed via [`Map::layers()`]."]
    Layer => LayerData
);

impl<'map> Layer<'map> {
    /// Get a reference to the layer's name.
    #[inline]
    pub fn name(&self) -> &str {
        self.data.name.as_ref()
    }

    /// Get the layer's id.
    #[inline]
    pub fn id(&self) -> u32 {
        self.data.id
    }

    /// Whether this layer should be visible or not.
    #[inline]
    pub fn visible(&self) -> bool {
        self.data.visible
    }

    /// Get the layer's x offset (in pixels).
    #[inline]
    pub fn offset_x(&self) -> f32 {
        self.data.offset_x
    }

    /// Get the layer's y offset (in pixels).
    #[inline]
    pub fn offset_y(&self) -> f32 {
        self.data.offset_y
    }

    /// Get the layer's x parallax factor.
    #[inline]
    pub fn parallax_x(&self) -> f32 {
        self.data.parallax_x
    }

    /// Get the layer's y parallax factor.
    #[inline]
    pub fn parallax_y(&self) -> f32 {
        self.data.parallax_y
    }

    /// Get the layer's opacity.
    #[inline]
    pub fn opacity(&self) -> f32 {
        self.data.opacity
    }

    /// Get the layer's tint color.
    #[inline]
    pub fn tint_color(&self) -> Option<Color> {
        self.data.tint_color
    }

    /// Get a reference to the layer's properties.
    #[inline]
    pub fn properties(&self) -> &Properties {
        &self.data.properties
    }

    /// Get the layer's type.
    #[inline]
    pub fn layer_type(&self) -> LayerType<'map> {
        LayerType::new(self.map, &self.data.layer_type)
    }
}

/// Represents some kind of map layer.
#[derive(Debug)]
pub enum LayerType<'map> {
    /// A tile layer; Also see [`TileLayer`].
    TileLayer(TileLayer<'map>),
    /// An object layer (also called object group); Also see [`ObjectLayer`].
    ObjectLayer(ObjectLayer<'map>),
    /// An image layer; Also see [`ImageLayer`].
    ImageLayer(ImageLayer<'map>),
    /// A group layer; Also see [`GroupLayer`].
    GroupLayer(GroupLayer<'map>),
}

impl<'map> LayerType<'map> {
    fn new(map: &'map Map, data: &'map LayerDataType) -> Self {
        match data {
            LayerDataType::TileLayer(data) => Self::TileLayer(TileLayer::new(map, data)),
            LayerDataType::ObjectLayer(data) => Self::ObjectLayer(ObjectLayer::new(map, data)),
            LayerDataType::ImageLayer(data) => Self::ImageLayer(ImageLayer::new(map, data)),
            LayerDataType::GroupLayer(data) => Self::GroupLayer(GroupLayer::new(map, data)),
        }
    }
}
