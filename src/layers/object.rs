use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    parse_properties,
    util::{get_attrs, map_wrapper, parse_tag, XmlEventResult},
    Color, Map, MapTilesetGid, Object, ObjectData, Properties, TiledError,
};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ObjectLayerData {
    objects: Vec<ObjectData>,
    colour: Option<Color>,
}

impl ObjectLayerData {
    /// If it is known that there are no objects with tile images in it (i.e. collision data)
    /// then we can pass in [`None`] as the tilesets
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tilesets: Option<&[MapTilesetGid]>,
    ) -> Result<(ObjectLayerData, Properties), TiledError> {
        let (c, ()) = get_attrs!(
            attrs,
            optionals: [
                ("color", colour, |v:String| v.parse().ok()),
            ],
            required: [],
            // this error should never happen since there are no required attrs
            TiledError::MalformedAttributes("object group parsing error".to_string())
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

map_wrapper!(ObjectLayer => ObjectLayerData);

impl<'map> ObjectLayer<'map> {
    pub fn get_object(&self, idx: usize) -> Option<Object<'map>> {
        self.data
            .objects
            .get(idx)
            .map(|data| Object::new(self.map, data))
    }

    pub fn objects(&self) -> Objects<'map> {
        Objects::new(self.map, self.data)
    }
}

/// An iterator that iterates over all the objects in an object layer, obtained via [`ObjectLayer::objects`].
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
    fn len(&self) -> usize {
        self.data.objects.len() - self.index
    }
}
