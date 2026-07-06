/// Loops through the attributes once and pulls out the ones we ask it to. It
/// will check that the required ones are there.
///
/// The syntax is:
/// ```ignore
/// get_attrs!(
///     for $attr in ($attributes) {
///         $($branch),*
///     }
///     $expression_to_return
/// )
/// ```
/// Where `$attributes` is anything that exposes `attributes()` for XML element attributes,
/// and `$attr` is the value of the attribute (a &str) going to be used in each branch. The
/// `$attributes` expression must be parenthesized to satisfy macro parsing.
///
/// Each branch indicates a variable to be set once a certain attribute is found.
/// Its syntax is as follows:
/// ```ignore
/// "attribute name" => variable_name = expression_using_$attr,
/// ```
///
/// For instance:
/// ```ignore
/// "source" => source = v.to_string(),
/// ```
/// The variable set has an inferred type `T`. In this case, `source` is inferred to be a `String`
/// because we call `to_string()`, and `$attr` has been named `v`.
///
/// If `Some` encapsulates the attribute name (like so: `Some("attribute name")`) then the attribute
/// is meant to be optional, which will make the variable an `Option<T>` rather than `T`. Even if it
/// is technically an Option, the assignment is still done *as if it was `T`*, for instance:
/// ```ignore
/// Some("name") => name = v.to_string(),
/// ```
///
/// Finally, branches can also use `?=` instead of `=`, which will make them accept a `Result<T, E>`
/// instead. If the expression results in an Err, the error will be handled internally and the
/// iteration will return early with a `Result<T, crate::Error>`.
///
/// Here are some examples of valid branches:
/// ```ignore
/// Some("spacing") => spacing ?= v.parse(),
/// Some("margin") => margin ?= v.parse(),
/// Some("columns") => columns ?= v.parse(),
/// Some("name") => name = v.to_string(),
///
/// "tilecount" => tilecount ?= v.parse::<u32>(),
/// "tilewidth" => tile_width ?= v.parse::<u32>(),
/// "tileheight" => tile_height ?= v.parse::<u32>(),
/// ```
///
/// Finally, after the `for` block, `$expression_to_return` indicates what to return once the
/// iteration has finished. It may refer to variables declared previously.
///
/// ## Example
/// ```ignore
/// let ((c, infinite), (v, o, w, h, tw, th)) = get_attrs!(
///     for v in (attrs) {
///         Some("backgroundcolor") => colour ?= v.parse(),
///         Some("infinite") => infinite = v == "1",
///         "version" => version = v.to_string(),
///         "orientation" => orientation ?= v.parse::<Orientation>(),
///         "width" => width ?= v.parse::<u32>(),
///         "height" => height ?= v.parse::<u32>(),
///         "tilewidth" => tile_width ?= v.parse::<u32>(),
///         "tileheight" => tile_height ?= v.parse::<u32>(),
///     }
///     ((colour, infinite), (version, orientation, width, height, tile_width, tile_height))
/// );
/// ```
macro_rules! get_attrs {
    (
        for $attr:ident in ($attrs:expr) {
            $($branches:tt)*
        }
        $ret_expr:expr
    ) => {
        {
            $crate::util::let_attr_branches!($($branches)*);

            for attr in ($attrs).attributes() {
                let attr =
                    attr.map_err(|err| $crate::Error::XmlDecodingError(err.into()))?;
                let local_name = attr.key.local_name();
                let __attr_name = local_name.as_ref();
                let __attr_value = attr
                    .unescape_value()
                    .map_err($crate::Error::XmlDecodingError)?;
                let $attr = __attr_value.as_ref();
                $crate::util::process_attr_branches!(__attr_name; $attr; $($branches)*);
            }

            $crate::util::handle_attr_branches!($($branches)*);

            $ret_expr
        }
    };
}

macro_rules! let_attr_branches {
    () => {};

    (Some($attr_pat_opt:literal) => $opt_var:ident $(?)?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        let mut $opt_var = None;
        $crate::util::let_attr_branches!($($($tail)*)?);
    };

    ($attr_pat_opt:literal => $opt_var:ident $(?)?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        let mut $opt_var = None;
        $crate::util::let_attr_branches!($($($tail)*)?);
    };
}

pub(crate) use let_attr_branches;

macro_rules! process_attr_branches {
    ($name:ident; $value:ident; ) => {};

    ($name:ident; $value:ident; Some($attr_pat_opt:literal) => $opt_var:ident = $opt_expr:expr $(, $($tail:tt)*)?) => {
        if $name == $attr_pat_opt.as_bytes() {
            $opt_var = Some($opt_expr);
        }
        else {
            $crate::util::process_attr_branches!($name; $value; $($($tail)*)?);
        }
    };

    ($name:ident; $value:ident; Some($attr_pat_opt:literal) => $opt_var:ident ?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        if $name == $attr_pat_opt.as_bytes() {
            $opt_var = Some($opt_expr.map_err(|_|
                $crate::Error::MalformedAttributes(
                    concat!("Error parsing optional attribute '", $attr_pat_opt, "'").to_owned()
                )
            )?);
        }
        else {
            $crate::util::process_attr_branches!($name; $value; $($($tail)*)?);
        }
    };

    ($name:ident; $value:ident; $attr_pat_opt:literal => $opt_var:ident = $opt_expr:expr $(, $($tail:tt)*)?) => {
        if $name == $attr_pat_opt.as_bytes() {
            $opt_var = Some($opt_expr);
        }
        else {
            $crate::util::process_attr_branches!($name; $value; $($($tail)*)?);
        }
    };

    ($name:ident; $value:ident; $attr_pat_opt:literal => $opt_var:ident ?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        if $name == $attr_pat_opt.as_bytes() {
            $opt_var = Some($opt_expr.map_err(|_|
                $crate::Error::MalformedAttributes(
                    concat!("Error parsing attribute '", $attr_pat_opt, "'").to_owned()
                )
            )?);
        }
        else {
            $crate::util::process_attr_branches!($name; $value; $($($tail)*)?);
        }
    }
}

pub(crate) use process_attr_branches;

macro_rules! handle_attr_branches {
    () => {};

    (Some($attr_pat_opt:literal) => $opt_var:ident $(?)?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        $crate::util::handle_attr_branches!($($($tail)*)?);
    };

    ($attr_pat_opt:literal => $opt_var:ident $(?)?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        let $opt_var = $opt_var
            .ok_or_else(||
                $crate::Error::MalformedAttributes(
                    concat!("Missing attribute: ", $attr_pat_opt).to_owned()
                )
            )?;

        $crate::util::handle_attr_branches!($($($tail)*)?);
    };
}

pub(crate) use handle_attr_branches;

/// Goes through the children of the tag and will call the correct function for
/// that child. Closes the tag.
macro_rules! parse_tag {
    ($elem:expr, {$($open_tag:expr => $open_method:expr),* $(,)*}) => {{
        let __elem = $elem;
        if !__elem.is_empty {
            let __reader = __elem.into_reader();
            let mut __event_buf = Vec::new();
            loop {
                let e = match __reader
                    .read_event_into(&mut __event_buf)
                    .map_err($crate::Error::XmlDecodingError)?
                {
                    quick_xml::events::Event::Start(e) => Some((e, false)),
                    quick_xml::events::Event::Empty(e) => Some((e, true)),
                    quick_xml::events::Event::End(_) => {
                        break;
                    }
                    quick_xml::events::Event::Eof => {
                        return Err($crate::Error::PrematureEnd(
                            "Document ended before we expected.".to_string(),
                        ));
                    }
                    _ => None,
                };

                if let Some((e, is_empty)) = e {
                    match e.local_name().as_ref() {
                        $(
                            name if name == $open_tag.as_bytes() => {
                                let mut __child =
                                    $crate::util::XmlElement::new(&mut *__reader, e, is_empty);
                                $open_method(__child)?;
                            }
                        )*
                        _ => {
                            if !is_empty {
                                let end = e.to_end().into_owned();
                                drop(e);
                                __reader
                                    .read_to_end_into(end.name(), &mut __event_buf)
                                    .map_err($crate::Error::XmlDecodingError)?;
                            }
                        }
                    }
                }
            }
        }
    }};
}

/// Creates a new type that wraps an internal data type over along with a map.
macro_rules! map_wrapper {
    ($(#[$attrs:meta])* $name:ident => $data_ty:ty) => {
        #[derive(Clone, Copy, PartialEq, Debug)]
        $(#[$attrs])*
        pub struct $name<'map> {
            pub(crate) map: &'map $crate::Map,
            pub(crate) data: &'map $data_ty,
        }

        impl<'map> $name<'map> {
            #[inline]
            pub(crate) fn new(map: &'map $crate::Map, data: &'map $data_ty) -> Self {
                Self { map, data }
            }

            /// Get the map this object is from.
            #[inline]
            pub fn map(&self) -> &'map $crate::Map {
                self.map
            }
        }

        impl<'map> std::ops::Deref for $name<'map> {
            type Target = $data_ty;

            #[inline]
            fn deref(&self) -> &'map Self::Target {
                self.data
            }
        }
    };
}

pub(crate) use get_attrs;
pub(crate) use map_wrapper;
pub(crate) use parse_tag;

use crate::{Gid, MapTilesetGid};

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

pub fn floor_div(a: i32, b: i32) -> i32 {
    let d = a / b;
    let r = a % b;

    if r == 0 {
        d
    } else {
        d - ((a < 0) ^ (b < 0)) as i32
    }
}
use std::io::BufRead;

use quick_xml::{Reader, events::BytesStart, events::Event};

use crate::{Error, Result};

pub(crate) struct XmlElement<'a, R: BufRead> {
    reader: &'a mut Reader<R>,
    pub attrs: BytesStart<'a>,
    pub is_empty: bool,
}

impl<'a, R: BufRead> XmlElement<'a, R> {
    pub(crate) fn new(reader: &'a mut Reader<R>, attrs: BytesStart<'a>, is_empty: bool) -> Self {
        Self {
            reader,
            attrs,
            is_empty,
        }
    }

    pub(crate) fn into_reader(self) -> &'a mut Reader<R> {
        self.reader
    }
}

pub(crate) fn read_text_or_cdata<R: BufRead, F, T>(
    elem: XmlElement<'_, R>,
    mut handler: F,
) -> Result<T>
where
    F: for<'a> FnMut(&'a str) -> Result<T>,
{
    if elem.is_empty {
        return handler("");
    }

    let reader = elem.into_reader();
    let mut event_buf = Vec::new();
    let mut result = None;
    loop {
        match reader
            .read_event_into(&mut event_buf)
            .map_err(Error::XmlDecodingError)?
        {
            Event::Text(e) if result.is_none() => {
                let unescaped = e.unescape().map_err(Error::XmlDecodingError)?;
                result = Some(Ok(handler(unescaped.as_ref())?));
            }
            Event::CData(e) if result.is_none() => {
                let unescaped = String::from_utf8_lossy(e.as_ref());
                result = Some(Ok(handler(unescaped.as_ref())?));
            }
            Event::Start(e) => {
                let end = e.to_end().into_owned();
                drop(e);
                reader
                    .read_to_end_into(end.name(), &mut event_buf)
                    .map_err(Error::XmlDecodingError)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => return result.unwrap_or_else(|| handler("")),
            Event::Eof => {
                return Err(Error::PrematureEnd(
                    "end of file while reading element contents".to_owned(),
                ));
            }
            _ => {}
        }
    }
}

pub(crate) fn parse_root_element<R, F, T>(
    reader: &mut Reader<R>,
    tag: &[u8],
    mut handler: F,
) -> Result<T>
where
    R: BufRead,
    F: FnMut(XmlElement<'_, R>) -> Result<T>,
{
    let mut event_buf = Vec::new();
    loop {
        let e = match reader
            .read_event_into(&mut event_buf)
            .map_err(Error::XmlDecodingError)?
        {
            Event::Start(e) => Some((e, false)),
            Event::Empty(e) => Some((e, true)),
            Event::End(e) => {
                return Err(Error::MalformedAttributes(format!(
                    "Unexpected closing tag </{}> before root element <{}>",
                    String::from_utf8_lossy(e.local_name().as_ref()),
                    String::from_utf8_lossy(tag),
                )));
            }
            Event::Eof => {
                return Err(Error::PrematureEnd(format!(
                    "Document ended before root element <{}> was found",
                    String::from_utf8_lossy(tag),
                )));
            }
            _ => None,
        };

        if let Some((e, empty)) = e {
            if e.local_name().as_ref() == tag {
                return handler(XmlElement::new(reader, e, empty));
            } else {
                return Err(Error::MalformedAttributes(format!(
                    "Expected root element <{}>, got <{}>",
                    String::from_utf8_lossy(tag),
                    String::from_utf8_lossy(e.local_name().as_ref()),
                )));
            }
        }
    }
}
