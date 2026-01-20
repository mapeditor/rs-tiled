use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{
    error::Result,
    layers::{LayerData, LayerTag},
    properties::{parse_properties, Properties},
    util::*,
    Layer, MapTilesetGid, ResourceCache, ResourceReader, Tileset,
};

/// The raw data of a [`GroupLayer`]. Does not include a reference to its parent [`Map`](crate::Map).
#[derive(Debug, PartialEq, Clone)]
pub struct GroupLayerData {
    layers: Vec<LayerData>,
}

impl GroupLayerData {
    pub(crate) fn new<R: std::io::BufRead>(
        mut elem: crate::util::XmlElement<'_, R>,
        infinite: bool,
        map_path: &Path,
        tilesets: &[MapTilesetGid],
        for_tileset: Option<Arc<Tileset>>,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<(Self, Properties)> {
        let mut properties = HashMap::new();
        let mut layers = Vec::new();
        elem.buf.clear();
        parse_tag!(&mut elem, {
            "layer" => |elem| {
                layers.push(LayerData::new(
                    elem,
                    LayerTag::Tiles,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset.as_ref().cloned(),
                    reader,
                    cache
                )?);
                Ok(())
            },
            "imagelayer" => |elem| {
                layers.push(LayerData::new(
                    elem,
                    LayerTag::Image,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset.as_ref().cloned(),
                    reader,
                    cache
                )?);
                Ok(())
            },
            "objectgroup" => |elem| {
                layers.push(LayerData::new(
                    elem,
                    LayerTag::Objects,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset.as_ref().cloned(),
                    reader,
                    cache
                )?);
                Ok(())
            },
            "group" => |elem| {
                layers.push(LayerData::new(
                    elem,
                    LayerTag::Group,
                    infinite,
                    map_path,
                    tilesets,
                    for_tileset.as_ref().cloned(),
                    reader,
                    cache
                )?);
                Ok(())
            },
            "properties" => |elem| {
                properties = parse_properties(elem)?;
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
    /// ## Example
    /// ```
    /// use tiled::Layer;
    /// # use tiled::Loader;
    ///
    /// # fn main() {
    /// # let map = Loader::new()
    /// #     .load_tmx_map("assets/tiled_group_layers.tmx")
    /// #     .unwrap();
    /// #
    /// let nested_layers: Vec<Layer> = map
    ///     .layers()
    ///     .filter_map(|layer| match layer.layer_type() {
    ///         tiled::LayerType::Group(layer) => Some(layer),
    ///         _ => None,
    ///     })
    ///     .flat_map(|layer| layer.layers())
    ///     .collect();
    ///
    /// dbg!(nested_layers);
    /// # }
    /// ```
    pub fn layers(&self) -> impl ExactSizeIterator<Item = Layer<'map>> + 'map {
        let map: &'map crate::Map = self.map;
        self.data
            .layers
            .iter()
            .map(move |layer| Layer::new(map, layer))
    }
    /// Gets a specific layer from the group by index.
    pub fn get_layer(&self, index: usize) -> Option<Layer<'map>> {
        self.data
            .layers
            .get(index)
            .map(|data| Layer::new(self.map, data))
    }
}
