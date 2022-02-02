use std::{collections::HashMap, io::Read, path::Path};

use xml::EventReader;

use crate::{parse_properties, util::parse_tag, Image, Properties, TiledError};

#[derive(Debug, PartialEq, Clone)]
pub struct ImageLayerData {
    pub image: Option<Image>,
}

impl ImageLayerData {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        map_path: &Path,
    ) -> Result<(Self, Properties), TiledError> {
        let mut image: Option<Image> = None;
        let mut properties = HashMap::new();

        let path_relative_to = map_path.parent().ok_or(TiledError::InvalidPath)?;

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
