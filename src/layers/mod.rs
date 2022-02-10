use std::path::Path;

use xml::attribute::OwnedAttribute;

use crate::{error::TiledError, properties::Properties, util::*, Map, MapTilesetGid, MapWrapper, Color};

mod image;
pub use image::*;
mod object;
pub use object::*;
mod tile;
pub use tile::*;
mod group;
pub use group::*;

#[derive(Clone, PartialEq, Debug)]
pub enum LayerDataType {
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
    pub name: String,
    pub id: u32,
    pub visible: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub opacity: f32,
    pub tint_color: Option<Color>,
    pub properties: Properties,
    pub layer_type: LayerDataType,
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

pub type Layer<'map> = MapWrapper<'map, LayerData>;

impl<'map> Layer<'map> {
    /// Get the layer's type.
    pub fn layer_type(&self) -> LayerType<'map> {
        LayerType::new(self.map(), &self.data().layer_type)
    }
}

pub enum LayerType<'map> {
    TileLayer(TileLayer<'map>),
    ObjectLayer(ObjectLayer<'map>),
    ImageLayer(ImageLayer<'map>),
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
