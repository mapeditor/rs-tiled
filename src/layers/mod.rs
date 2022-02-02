use std::{io::Read, path::Path};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{error::TiledError, properties::Properties, util::*, Map};

mod image;
pub use image::*;
mod object;
pub use object::*;
mod tile;
pub use tile::*;

#[derive(Clone, PartialEq, Debug)]
pub enum LayerDataType {
    TileLayer(TileLayerData),
    ObjectLayer(ObjectLayerData),
    ImageLayer(ImageLayerData),
    // TODO: Support group layers
}

#[derive(Clone, Copy)]
pub(crate) enum LayerTag {
    TileLayer,
    ObjectLayer,
    ImageLayer,
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
    pub properties: Properties,
    pub layer_type: LayerDataType,
}

impl LayerData {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        tag: LayerTag,
        infinite: bool,
        map_path: &Path,
    ) -> Result<Self, TiledError> {
        let ((opacity, visible, offset_x, offset_y, parallax_x, parallax_y, name, id), ()) = get_attrs!(
            attrs,
            optionals: [
                ("opacity", opacity, |v:String| v.parse().ok()),
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
                let (ty, properties) = TileLayerData::new(parser, attrs, infinite)?;
                (LayerDataType::TileLayer(ty), properties)
            }
            LayerTag::ObjectLayer => {
                let (ty, properties) = ObjectLayerData::new(parser, attrs)?;
                (LayerDataType::ObjectLayer(ty), properties)
            }
            LayerTag::ImageLayer => {
                let (ty, properties) = ImageLayerData::new(parser, map_path)?;
                (LayerDataType::ImageLayer(ty), properties)
            }
        };

        Ok(Self {
            visible: visible.unwrap_or(true),
            offset_x: offset_x.unwrap_or(0.0),
            offset_y: offset_y.unwrap_or(0.0),
            parallax_x: parallax_x.unwrap_or(1.0),
            parallax_y: parallax_y.unwrap_or(1.0),
            opacity: opacity.unwrap_or(1.0),
            name: name.unwrap_or_default(),
            id: id.unwrap_or(0),
            properties,
            layer_type: ty,
        })
    }
}

/// A wrapper over a naive layer-related type that holds a reference to the parent map as well as the layer data.
#[derive(Clone, PartialEq, Debug)]
pub struct LayerWrapper<'map, DataT>
where
    DataT: Clone + PartialEq + std::fmt::Debug,
{
    map: &'map Map,
    data: &'map DataT,
}

impl<'map, DataT> LayerWrapper<'map, DataT>
where
    DataT: Clone + PartialEq + std::fmt::Debug,
{
    pub(crate) fn new(map: &'map Map, data: &'map DataT) -> Self {
        Self { map, data }
    }

    /// Get the layer's data.
    pub fn data(&self) -> &DataT {
        self.data
    }

    /// Get the layer's map.
    pub fn map(&self) -> &Map {
        self.map
    }
}

pub type Layer<'map> = LayerWrapper<'map, LayerData>;

impl<'map> Layer<'map> {
    /// Get the layer's type.
    pub fn layer_type(&self) -> LayerType {
        LayerType::new(self.map, &self.data.layer_type)
    }
}

pub enum LayerType<'map> {
    TileLayer(TileLayer<'map>),
    ObjectLayer(ObjectLayer<'map>),
    ImageLayer(ImageLayer<'map>),
    // TODO: Support group layers
}

impl<'map> LayerType<'map> {
    fn new(map: &'map Map, data: &'map LayerDataType) -> Self {
        match data {
            LayerDataType::TileLayer(data) => Self::TileLayer(TileLayer::new(map, data)),
            LayerDataType::ObjectLayer(data) => Self::ObjectLayer(ObjectLayer::new(map, data)),
            LayerDataType::ImageLayer(data) => Self::ImageLayer(ImageLayer::new(map, data)),
        }
    }
}
