use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    parse_properties,
    util::{get_attrs, map_wrapper, parse_tag, XmlEventResult},
    Color, Error, Map, MapTilesetGid, Object, ObjectData, Properties,
};

/// Raw data referring to a map object layer or tile collision data.
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectLayerData {
    /// The objects present in this layer.
    pub objects: Vec<ObjectData>,
    /// The color used in the editor to display objects in this layer.
    pub colour: Option<Color>,
}

impl ObjectLayerData {
    /// If it is known that there are no objects with tile images in it (i.e. collision data)
    /// then we can pass in [`None`] as the tilesets
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tilesets: Option<&[MapTilesetGid]>,
    ) -> Result<(ObjectLayerData, Properties), Error> {
        let c = get_attrs!(
            attrs,
            optionals: [
                ("color", colour, |v:String| v.parse().ok()),
            ]
        );
        let mut objects = Vec::new();
        let mut properties = HashMap::new();
        parse_tag!(parser, "objectgroup", {
            "object" => |attrs| {
                objects.push(ObjectData::new(parser, attrs, tilesets)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });
        Ok((ObjectLayerData { objects, colour: c }, properties))
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
    pub fn objects(&self) -> Objects<'map> {
        Objects::new(self.map, self.data)
    }

    /// Get a reference to the object layer's colour.
    pub fn colour(&self) -> Option<Color> {
        self.data.colour
    }
}

/// An iterator that iterates over all the objects in an object layer, obtained via [`ObjectLayer::objects`].
#[derive(Debug)]
pub struct Objects<'map> {
    map: &'map Map,
    data: &'map ObjectLayerData,
    index: usize,
}

impl<'map> Objects<'map> {
    fn new(map: &'map Map, data: &'map ObjectLayerData) -> Self {
        Self {
            map,
            data,
            index: 0,
        }
    }
}

impl<'map> Iterator for Objects<'map> {
    type Item = Object<'map>;

    fn next(&mut self) -> Option<Self::Item> {
        let object_data = self.data.objects.get(self.index)?;
        self.index += 1;
        Some(Object::new(self.map, object_data))
    }
}

impl<'map> ExactSizeIterator for Objects<'map> {
    #[inline]
    fn len(&self) -> usize {
        self.data.objects.len() - self.index
    }
}
