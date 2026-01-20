use std::{collections::HashMap, path::Path};

use crate::{
    parse_properties,
    util::{get_attrs, map_wrapper, parse_tag},
    Error, Image, Properties, Result,
};

/// The raw data of an [`ImageLayer`]. Does not include a reference to its parent [`Map`](crate::Map).
#[derive(Debug, PartialEq, Clone)]
pub struct ImageLayerData {
    /// The single image this layer contains, if it exists.
    pub image: Option<Image>,
    /// The layer's x repeat factor (true = repeat, false = no repeat).
    pub repeat_x: bool,
    /// The layer's y repeat factor (true = repeat, false = no repeat).
    pub repeat_y: bool,
}

impl ImageLayerData {
    pub(crate) fn new(
        xml_reader: &mut quick_xml::Reader<impl std::io::BufRead>,
        buf: &mut Vec<u8>,
        attrs: quick_xml::events::BytesStart<'_>,
        map_path: &Path,
    ) -> Result<(Self, Properties)> {
        let mut image: Option<Image> = None;
        let mut properties = HashMap::new();

        let path_relative_to = map_path.parent().ok_or(Error::PathIsNotFile)?;

        // Parse repeat attributes from the imagelayer tag
        let (repeat_x, repeat_y) = get_attrs!(
            for v in attrs {
                Some("repeatx") => repeat_x ?= v.parse::<i32>().map(|val| val == 1),
                Some("repeaty") => repeat_y ?= v.parse::<i32>().map(|val| val == 1),
            }
            (repeat_x, repeat_y)
        );
        buf.clear();

        parse_tag!(xml_reader, buf, "imagelayer", {
            "image" => |attrs| {
                image = Some(Image::new(xml_reader, buf, attrs, path_relative_to)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(xml_reader, buf)?;
                Ok(())
            },
        });
        Ok((
            ImageLayerData {
                image,
                repeat_x: repeat_x.unwrap_or(false),
                repeat_y: repeat_y.unwrap_or(false),
            },
            properties,
        ))
    }
}

map_wrapper!(
    #[doc = "A layer consisting of a single image."]
    #[doc = "\nAlso see the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#imagelayer)."]
    ImageLayer => ImageLayerData
);
