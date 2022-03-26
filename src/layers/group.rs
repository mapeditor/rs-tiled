use std::collections::HashMap;
use std::path::Path;

use crate::{
    error::Result,
    layers::{LayerData, LayerTag},
    map::MapTilesetGid,
    properties::{parse_properties, Properties},
    util::*,
    Error, Layer,
};

/// The raw data of a [`GroupLayer`]. Does not include a reference to its parent [`Map`](crate::Map).
#[derive(Debug, PartialEq, Clone)]
pub struct GroupLayerData {
    layers: Vec<LayerData>,
}

impl GroupLayerData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        infinite: bool,
        map_path: &Path,
        tilesets: &[MapTilesetGid],
    ) -> Result<(Self, Properties)> {
        let mut properties = HashMap::new();
        let mut layers = Vec::new();
        parse_tag!(parser, "group", {
            "layer" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::Tile,
                    infinite,
                    map_path,
                    &tilesets,
                )?);
                Ok(())
            },
            "imagelayer" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::Image,
                    infinite,
                    map_path,
                    &tilesets,
                )?);
                Ok(())
            },
            "objectgroup" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::Object,
                    infinite,
                    map_path,
                    &tilesets,
                )?);
                Ok(())
            },
            "group" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::Group,
                    infinite,
                    map_path,
                    &tilesets,
                )?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });
        Ok((Self { layers }, properties))
    }
}

map_wrapper!(
    #[doc = "A group layer, used to organize the layers of the map in a hierarchy."]
    #[doc = "\nAlso see the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#group)."]
    #[doc = "## Note"]
    #[doc = "In Tiled, the properties of the group layer recursively affect child layers.
    Implementing this behavior is left up to the user of this library."]
    GroupLayer => GroupLayerData
);

impl<'map> GroupLayer<'map> {
    /// Returns an iterator over the layers present in this group in display order.
    pub fn layers(&self) -> impl ExactSizeIterator<Item = Layer> {
        self.layers
            .iter()
            .map(move |layer| Layer::new(self.map, layer))
    }
    /// Gets a specific layer from the group by index.
    pub fn get_layer(&self, index: usize) -> Option<Layer> {
        self.data
            .layers
            .get(index)
            .map(|data| Layer::new(self.map, data))
    }
}
