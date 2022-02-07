use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    parse_properties,
    util::{get_attrs, parse_tag, XmlEventResult},
    Color, MapTileset, Object, ObjectData, Properties, TiledError, TiledWrapper,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectLayerData {
    pub objects: Vec<ObjectData>,
    pub colour: Option<Color>,
}

impl ObjectLayerData {
    /// If it is known that there are no objects with tile images in it (i.e. collision data)
    /// then we can pass in [`None`] as the tilesets
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tilesets: Option<&[MapTileset]>,
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

pub type ObjectLayer<'map> = TiledWrapper<'map, ObjectLayerData>;

impl<'map> ObjectLayer<'map> {
    pub fn get_object(&self, idx: usize) -> Option<Object<'map>> {
        self.data()
            .objects
            .get(idx)
            .map(|data| Object::new(self.map(), data))
    }
}
