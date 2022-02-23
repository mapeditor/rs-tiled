use xml::attribute::OwnedAttribute;

use crate::{error::TiledError, util::{get_attrs, XmlEventResult, parse_tag}};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Frame {
    pub tile_id: u32,
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