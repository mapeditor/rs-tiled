//! Structures related to tile animations.

use xml::attribute::OwnedAttribute;

use crate::{
    error::TiledError,
    util::{get_attrs, parse_tag, XmlEventResult},
};

/// A structure describing a [frame] of a [TMX tile animation].
///
/// [frame]: https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tmx-frame
/// [TMX tile animation]: https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#animation
#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
    /// The local ID of a tile within the parent tileset.
    pub tile_id: u32,
    /// How long (in milliseconds) this frame should be displayed before advancing to the next frame.
    pub duration: u32,
}

impl Frame {
    pub(crate) fn new(attrs: Vec<OwnedAttribute>) -> Result<Frame, TiledError> {
        let ((), (tile_id, duration)) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("tileid", tile_id, |v:String| v.parse().ok()),
                ("duration", duration, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("A frame must have tileid and duration".to_string())
        );
        Ok(Frame {
            tile_id: tile_id,
            duration: duration,
        })
    }
}

pub(crate) fn parse_animation(
    parser: &mut impl Iterator<Item = XmlEventResult>,
) -> Result<Vec<Frame>, TiledError> {
    let mut animation = Vec::new();
    parse_tag!(parser, "animation", {
        "frame" => |attrs| {
            animation.push(Frame::new(attrs)?);
            Ok(())
        },
    });
    Ok(animation)
}
