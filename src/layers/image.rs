use std::{collections::HashMap, path::Path};

use crate::{
    parse_properties,
    util::{parse_tag, XmlEventResult, map_wrapper},
    Image, Properties, TiledError,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ImageLayerData {
    image: Option<Image>,
}

impl ImageLayerData {
    /// Get a reference to the image layer's image.
    #[inline]
    pub fn image(&self) -> Option<&Image> {
        self.image.as_ref()
    }

    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        map_path: &Path,
    ) -> Result<(Self, Properties), TiledError> {
        let mut image: Option<Image> = None;
        let mut properties = HashMap::new();

        let path_relative_to = map_path.parent().ok_or(TiledError::PathIsNotFile)?;

        parse_tag!(parser, "imagelayer", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, path_relative_to)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });
        Ok((ImageLayerData { image }, properties))
    }
}

map_wrapper!(
    #[doc = "A layer consisting of a single image."]
    #[doc = "\nAlso see the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#imagelayer)."]
    ImageLayer => ImageLayerData
);
