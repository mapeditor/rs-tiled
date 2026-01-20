//! Structures related to tile animations.

use crate::{
    error::Result,
    util::{get_attrs, parse_tag, XmlElement},
};

/// A structure describing a [frame] of a [TMX tile animation].
///
/// [frame]: https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tmx-frame
/// [TMX tile animation]: https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#animation
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Frame {
    /// The local ID of a tile within the parent tileset.
    pub tile_id: u32,
    /// How long (in milliseconds) this frame should be displayed before advancing to the next frame.
    pub duration: u32,
}

impl Frame {
    pub(crate) fn new<R: std::io::BufRead>(mut elem: XmlElement<'_, R>) -> Result<Frame> {
        let (tile_id, duration) = get_attrs!(
            for v in (elem.attrs) {
                "tileid" => tile_id ?= v.parse::<u32>(),
                "duration" => duration ?= v.parse::<u32>(),
            }
            (tile_id, duration)
        );
        parse_tag!(&mut elem, {});
        Ok(Frame { tile_id, duration })
    }
}

pub(crate) fn parse_animation<R: std::io::BufRead>(
    mut elem: XmlElement<'_, R>,
) -> Result<Vec<Frame>> {
    let mut animation = Vec::new();
    elem.buf.clear();
    parse_tag!(&mut elem, {
        "frame" => |elem| {
            animation.push(Frame::new(elem)?);
            Ok(())
        },
    });
    Ok(animation)
}
