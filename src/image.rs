use std::path::{Path, PathBuf};

use xml::attribute::OwnedAttribute;

use crate::{error::TiledError, properties::Color, util::*};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Image {
    /// The filepath of the image.
    ///
    /// ## Note
    /// The crate does not currently support embedded images (Even though Tiled
    /// does not allow creating maps with embedded image data, the TMX format does; [source])
    ///
    /// [source]: https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#image
    // TODO: Embedded images
    pub source: PathBuf,
    pub width: i32,
    pub height: i32,
    pub transparent_colour: Option<Color>,
}

impl Image {
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        path_relative_to: impl AsRef<Path>,
    ) -> Result<Image, TiledError> {
        let (c, (s, w, h)) = get_attrs!(
            attrs,
            optionals: [
                ("trans", trans, |v:String| v.parse().ok()),
            ],
            required: [
                ("source", source, |v| Some(v)),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("Image must have a source, width and height with correct types".to_string())
        );

        parse_tag!(parser, "image", { "" => |_| Ok(()) });
        Ok(Image {
            source: path_relative_to.as_ref().join(s),
            width: w,
            height: h,
            transparent_colour: c,
        })
    }
}
