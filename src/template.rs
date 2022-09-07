use std::path::Path;
use std::sync::Arc;

use xml::EventReader;
use xml::{attribute::OwnedAttribute, reader::XmlEvent};

use crate::parse;
use crate::parse::common::tileset::EmbeddedParseResultType;
use crate::{
    util::*, Error, MapTilesetGid, ObjectData, ResourceCache, ResourceReader, Result, Tileset,
};

/// A template, consisting of an object and a tileset
///
/// Templates define a tileset and object data to use for an object that can be shared between multiple objects and
/// maps.
#[derive(Clone, Debug)]
pub struct Template {
    /// The tileset this template contains a reference to
    pub tileset: Option<Arc<Tileset>>,
    /// The object data for this template
    pub object: ObjectData,
}
