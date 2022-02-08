/// Loops through the attributes once and pulls out the ones we ask it to. It
/// will check that the required ones are there. This could have been done with
/// attrs.find but that would be inefficient.
///
/// This is probably a really terrible way to do this. It does cut down on lines
/// though which is nice.
macro_rules! get_attrs {
    ($attrs:expr, optionals: [$(($oName:pat, $oVar:ident, $oMethod:expr)),* $(,)*],
     required: [$(($name:pat, $var:ident, $method:expr)),* $(,)*], $err:expr) => {
        {
            $(let mut $oVar = None;)*
            $(let mut $var = None;)*
            for attr in $attrs.iter() {
                match attr.name.local_name.as_ref() {
                    $($oName => $oVar = $oMethod(attr.value.clone()),)*
                    $($name => $var = $method(attr.value.clone()),)*
                    _ => {}
                }
            }
            if !(true $(&& $var.is_some())*) {
                return Err($err);
            }
            (($($oVar),*), ($($var.unwrap()),*))
        }
    }
}

/// Goes through the children of the tag and will call the correct function for
/// that child. Closes the tag.
macro_rules! parse_tag {
    ($parser:expr, $close_tag:expr, {$($open_tag:expr => $open_method:expr),* $(,)*}) => {
        while let Some(next) = $parser.next() {
            match next.map_err(TiledError::XmlDecodingError)? {
                xml::reader::XmlEvent::StartElement {name, attributes, ..} => {
                    if false {}
                    $(else if name.local_name == $open_tag {
                        match $open_method(attributes) {
                            Ok(()) => {},
                            Err(e) => return Err(e)
                        };
                    })*
                }
                xml::reader::XmlEvent::EndElement {name, ..} => {
                    if name.local_name == $close_tag {
                        break;
                    }
                }
                xml::reader::XmlEvent::EndDocument => return Err(TiledError::PrematureEnd("Document ended before we expected.".to_string())),
                _ => {}
            }
        }
    }
}

pub(crate) use get_attrs;
pub(crate) use parse_tag;

use crate::{animation::Frame, error::TiledError, Gid, MapTileset, MapTilesetGid};

// TODO: Move to animation module
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

pub(crate) type XmlEventResult = xml::reader::Result<xml::reader::XmlEvent>;

/// Returns both the tileset and its index
pub(crate) fn get_tileset_for_gid(
    tilesets: &[MapTilesetGid],
    gid: Gid,
) -> Option<(usize, &MapTilesetGid)> {
    tilesets
        .iter()
        .enumerate()
        .rev()
        .find(|(_idx, ts)| ts.first_gid <= gid)
}
