use std::{collections::HashMap, path::Path};

use crate::{
    parse_properties,
    util::{parse_tag, XmlEventResult, map_wrapper},
    Image, Properties, TiledError,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ImageLayerData {
    pub image: Option<Image>,
}

impl ImageLayerData {
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

map_wrapper!(ImageLayer => ImageLayerData);

impl<'map> ImageLayer<'map> {
    /// Get a reference to the image layer's image.
    pub fn image(&self) -> Option<&Image> {
        self.data.image.as_ref()
    }
}
