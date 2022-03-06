use std::path::{Path, PathBuf};

use xml::attribute::OwnedAttribute;

use crate::{error::TiledError, properties::Color, util::*};

/// A reference to an image stored somewhere within the filesystem.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Image {
    /// The **uncanonicalized** filepath of the image, starting from the path given to load the file
    /// this image is in. See the example for more details.
    ///
    /// ## Note
    /// The crate does not currently support embedded images (Even though Tiled
    /// does not allow creating maps with embedded image data, the TMX format does; [source])
    ///
    /// Currently, the crate is not prepared to handle anything but OS paths. Using VFS is a hard
    /// task that involves a lot of messy path manipulation. [Tracking issue]
    ///
    /// [source]: https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#image
    /// [Tracking issue]: https://github.com/mapeditor/rs-tiled/issues/37
    ///
    /// ## Example
    /// ```
    /// use std::path::Path;
    /// use std::fs::File;
    /// use tiled::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let map = Map::parse_file(
    ///     "assets/folder/tiled_relative_paths.tmx",
    ///     &mut FilesystemResourceCache::new(),
    /// )?;
    ///
    /// let image_layer = match map
    ///     .layers()
    ///     .find(|layer| layer.name() == "image")
    ///     .unwrap()
    ///     .layer_type()
    /// {
    ///     LayerType::ImageLayer(layer) => layer,
    ///     _ => panic!(),
    /// };
    ///
    /// // Image layer has an image with the source attribute set to "../tilesheet.png"
    /// // Given the information we gave to the `parse_file` function, the image source should be
    /// // "assets/folder/../tilesheet.png". The filepath is not canonicalized.
    /// let image_source = &image_layer.image().unwrap().source;
    ///
    /// assert_eq!(
    ///     image_source,
    ///     Path::new("assets/folder/../tilesheet.png")
    /// );
    ///
    /// // If you are using the OS's filesystem, figuring out the real path of the image is as easy
    /// // as canonicalizing the path. If you are using some sort of VFS, this task is much harder
    /// // since std::path is meant to be used with the OS. This will be fixed in the future!
    /// let image_source = image_source.canonicalize()?;
    /// assert!(File::open(image_source).is_ok());
    /// # Ok(())
    /// # }
    /// ```
    /// Check the assets/tiled_relative_paths.tmx file at the crate root to see the structure of the
    /// file this example is referring to.
    // TODO: Embedded images
    // TODO: Figure out how to serve crate users paths in a better way
    pub source: PathBuf,
    /// The width in pixels of the image.
    pub width: i32,
    /// The height in pixels of the image.
    pub height: i32,
    /// A color that should be interpreted as transparent (0 alpha), if any.
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
