use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{
    error::TiledError,
    layers::{LayerData, LayerTag},
    properties::{parse_properties, Properties},
    util::*,
    Layer, Map, MapTilesetGid, ResourceCache, Tileset,
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
        for_tileset: Option<Arc<Tileset>>,
        cache: &mut impl ResourceCache,
    ) -> Result<(Self, Properties), TiledError> {
        let mut properties = HashMap::new();
        let mut layers = Vec::new();
        parse_tag!(parser, "group", {
            "layer" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::TileLayer,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset.as_ref().cloned(),
                    cache
                )?);
                Ok(())
            },
            "imagelayer" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::ImageLayer,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset.as_ref().cloned(),
                    cache
                )?);
                Ok(())
            },
            "objectgroup" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::ObjectLayer,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset.as_ref().cloned(),
                    cache
                )?);
                Ok(())
            },
            "group" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::GroupLayer,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset.as_ref().cloned(),
                    cache
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
    pub fn layers(&self) -> GroupLayerIter {
        GroupLayerIter::new(self.map, self.data)
    }
    /// Gets a specific layer from the group by index.
    pub fn get_layer(&self, index: usize) -> Option<Layer> {
        self.data
            .layers
            .get(index)
            .map(|data| Layer::new(self.map, data))
    }
}

/// An iterator that iterates over all the layers in a group layer, obtained via [`GroupLayer::layers`].
#[derive(Debug)]
pub struct GroupLayerIter<'map> {
    map: &'map Map,
    group: &'map GroupLayerData,
    index: usize,
}

impl<'map> GroupLayerIter<'map> {
    fn new(map: &'map Map, group: &'map GroupLayerData) -> Self {
        Self {
            map,
            group,
            index: 0,
        }
    }
}

impl<'map> Iterator for GroupLayerIter<'map> {
    type Item = Layer<'map>;
    fn next(&mut self) -> Option<Self::Item> {
        let layer_data = self.group.layers.get(self.index)?;
        self.index += 1;
        Some(Layer::new(self.map, layer_data))
    }
}

impl<'map> ExactSizeIterator for GroupLayerIter<'map> {
    fn len(&self) -> usize {
        self.group.layers.len() - self.index
    }
}
