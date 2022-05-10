use std::{path::Path, sync::Arc};

use xml::attribute::OwnedAttribute;

use crate::{
    error::Result, properties::Properties, util::*, Color, Map, MapTilesetGid, ResourceCache,
    ResourceReader, Tileset,
};

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
    Tiles(TileLayerData),
    Objects(ObjectLayerData),
    Image(ImageLayerData),
    Group(GroupLayerData),
}

#[derive(Clone, Copy)]
pub(crate) enum LayerTag {
    Tiles,
    Objects,
    Image,
    Group,
}

/// The raw data of a [`Layer`]. Does not include a reference to its parent [`Map`](crate::Map).
#[derive(Clone, PartialEq, Debug)]
pub struct LayerData {
    /// The layer's name, set arbitrarily by the user.
    pub name: String,
    id: u32,
    /// Whether this layer should be visible or not.
    pub visible: bool,
    /// The layer's x offset (in pixels).
    pub offset_x: f32,
    /// The layer's y offset (in pixels).
    pub offset_y: f32,
    /// The layer's x parallax factor.
    pub parallax_x: f32,
    /// The layer's y parallax factor.
    pub parallax_y: f32,
    /// The layer's opacity.
    pub opacity: f32,
    /// The layer's tint color.
    pub tint_color: Option<Color>,
    /// The layer's custom properties, as arbitrarily set by the user.
    pub properties: Properties,
    layer_type: LayerDataType,
}

impl LayerData {
    /// Get the layer's id. Unique within the parent map. Valid only if greater than 0. Defaults to
    /// 0 if the layer was loaded from a file that didn't have the attribute present.
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tag: LayerTag,
        infinite: bool,
        map_path: &Path,
        tilesets: &[MapTilesetGid],
        for_tileset: Option<Arc<Tileset>>,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Self> {
        let (opacity, tint_color, visible, offset_x, offset_y, parallax_x, parallax_y, name, id) = get_attrs!(
            attrs,
            optionals: [
                ("opacity", opacity, |v:String| v.parse().ok()),
                ("tintcolor", tint_color, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("offsetx", offset_x, |v:String| v.parse().ok()),
                ("offsety", offset_y, |v:String| v.parse().ok()),
                ("parallaxx", parallax_x, |v:String| v.parse().ok()),
                ("parallaxy", parallax_y, |v:String| v.parse().ok()),
                ("name", name, Some),
                ("id", id, |v:String| v.parse().ok()),
            ]
        );

        let (ty, properties) = match tag {
            LayerTag::Tiles => {
                let (ty, properties) = TileLayerData::new(parser, attrs, infinite, tilesets)?;
                (LayerDataType::Tiles(ty), properties)
            }
            LayerTag::Objects => {
                let (ty, properties) = ObjectLayerData::new(
                    parser,
                    attrs,
                    Some(tilesets),
                    for_tileset,
                    map_path,
                    reader,
                    cache,
                )?;
                (LayerDataType::Objects(ty), properties)
            }
            LayerTag::Image => {
                let (ty, properties) = ImageLayerData::new(parser, map_path)?;
                (LayerDataType::Image(ty), properties)
            }
            LayerTag::Group => {
                let (ty, properties) = GroupLayerData::new(
                    parser,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset,
                    reader,
                    cache,
                )?;
                (LayerDataType::Group(ty), properties)
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
    Tiles(TileLayer<'map>),
    /// An object layer (also called object group); Also see [`ObjectLayer`].
    Objects(ObjectLayer<'map>),
    /// An image layer; Also see [`ImageLayer`].
    Image(ImageLayer<'map>),
    /// A group layer; Also see [`GroupLayer`].
    Group(GroupLayer<'map>),
}

impl<'map> LayerType<'map> {
    fn new(map: &'map Map, data: &'map LayerDataType) -> Self {
        match data {
            LayerDataType::Tiles(data) => Self::Tiles(TileLayer::new(map, data)),
            LayerDataType::Objects(data) => Self::Objects(ObjectLayer::new(map, data)),
            LayerDataType::Image(data) => Self::Image(ImageLayer::new(map, data)),
            LayerDataType::Group(data) => Self::Group(GroupLayer::new(map, data)),
        }
    }
}
