use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{
    parse_properties,
    util::{get_attrs, map_wrapper, parse_tag},
    Color, MapTilesetGid, Object, ObjectData, Properties, ResourceCache, ResourceReader, Result,
    Tileset,
};

/// The order in which the objects of an object layer are drawn.
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum DrawOrder {
    /// The objects are drawn sorted by their y-coordinate.
    #[default]
    TopDown,
    /// The objects are drawn in the order of appearance in the map file, which can be manually
    /// arranged in the editor.
    Index,
}

#[derive(Debug)]
/// An error arising from trying to parse a [`DrawOrder`] that is not valid.
pub struct DrawOrderParseError {
    /// The invalid string found.
    pub str_found: String,
}

impl std::fmt::Display for DrawOrderParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "failed to parse draw order, valid options are `topdown` and `index` \
        but got `{}` instead",
            self.str_found
        ))
    }
}

impl std::str::FromStr for DrawOrder {
    type Err = DrawOrderParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "topdown" => Ok(DrawOrder::TopDown),
            "index" => Ok(DrawOrder::Index),
            _ => Err(DrawOrderParseError {
                str_found: s.to_owned(),
            }),
        }
    }
}

impl std::fmt::Display for DrawOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawOrder::TopDown => write!(f, "topdown"),
            DrawOrder::Index => write!(f, "index"),
        }
    }
}

/// Raw data referring to a map object layer or tile collision data.
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectLayerData {
    objects: Vec<ObjectData>,
    /// The color used in the editor to display objects in this layer.
    pub colour: Option<Color>,
    /// The order in which the objects in this layer are drawn.
    pub draw_order: DrawOrder,
}

impl ObjectLayerData {
    /// If it is known that there are no objects with tile images in it (i.e. collision data)
    /// then we can pass in [`None`] as the tilesets
    pub(crate) fn new<R: std::io::BufRead>(
        elem: crate::util::XmlElement<'_, R>,
        tilesets: Option<&[MapTilesetGid]>,
        for_tileset: Option<Arc<Tileset>>,
        // path_relative_to is a directory to which all other files are relative to
        path_relative_to: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<(ObjectLayerData, Properties)> {
        let (c, draw_order) = get_attrs!(
            for v in (elem.attrs) {
                Some("color") => color ?= v.parse(),
                Some("draworder") => draw_order ?= v.parse::<DrawOrder>(),
            }
            (color, draw_order)
        );
        let draw_order = draw_order.unwrap_or_default();
        let mut objects = Vec::new();
        let mut properties = HashMap::new();
        parse_tag!(elem, {
            "object" => |elem| {
                objects.push(ObjectData::new(elem, tilesets, for_tileset.as_ref().cloned(), path_relative_to, reader, cache)?);
                Ok(())
            },
            "properties" => |elem| {
                properties = parse_properties(elem)?;
                Ok(())
            },
        });
        Ok((
            ObjectLayerData {
                objects,
                colour: c,
                draw_order,
            },
            properties,
        ))
    }

    /// Returns the data belonging to the objects contained within the layer, in the order they were
    /// declared in the TMX file.
    #[inline]
    pub fn object_data(&self) -> &[ObjectData] {
        self.objects.as_ref()
    }
}

map_wrapper!(
    #[doc = "Also called an \"object group\". Used for storing [`Object`]s in a map."]
    ObjectLayer => ObjectLayerData);

impl<'map> ObjectLayer<'map> {
    /// Obtains the object corresponding to the index given.
    pub fn get_object(&self, idx: usize) -> Option<Object<'map>> {
        self.data
            .objects
            .get(idx)
            .map(|data| Object::new(self.map, data))
    }

    /// Returns an iterator over the objects present in this layer, in the order they were declared
    /// in in the TMX file.
    ///
    /// ## Example
    /// ```
    /// # use tiled::Loader;
    /// use tiled::Object;
    ///
    /// # fn main() {
    /// # let map = Loader::new()
    /// #     .load_tmx_map("assets/tiled_group_layers.tmx")
    /// #     .unwrap();
    /// #
    /// let spawnpoints: Vec<Object> = map
    ///     .layers()
    ///     .filter_map(|layer| match layer.layer_type() {
    ///         tiled::LayerType::Objects(layer) => Some(layer),
    ///         _ => None,
    ///     })
    ///     .flat_map(|layer| layer.objects())
    ///     .filter(|object| object.user_type == "spawn")
    ///     .collect();
    ///
    /// dbg!(spawnpoints);
    /// # }
    /// ```
    #[inline]
    pub fn objects(&self) -> impl ExactSizeIterator<Item = Object<'map>> + 'map {
        let map: &'map crate::Map = self.map;
        self.data
            .objects
            .iter()
            .map(move |object| Object::new(map, object))
    }
}
