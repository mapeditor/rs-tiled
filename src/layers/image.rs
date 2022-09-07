use std::{collections::HashMap, path::Path};

use crate::{
    util::{map_wrapper, parse_tag, XmlEventResult},
    Error, Image, Properties, Result, parse::xml::properties::parse_properties,
};

/// The raw data of an [`ImageLayer`]. Does not include a reference to its parent [`Map`](crate::Map).
#[derive(Debug, PartialEq, Clone)]
pub struct ImageLayerData {
    /// The single image this layer contains, if it exists.
    pub image: Option<Image>,
}

impl ImageLayerData {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        map_path: &Path,
    ) -> Result<(Self, Properties)> {
        let mut image: Option<Image> = None;
        let mut properties = HashMap::new();

        let path_relative_to = map_path.parent().ok_or(Error::PathIsNotFile)?;

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
