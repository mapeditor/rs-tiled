use std::{collections::HashMap, fmt, fs::File, io::Read, path::Path, str::FromStr};

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    error::{ParseTileError, TiledError},
    layers::{Layer, LayerTag},
    properties::{parse_properties, Color, Properties},
    tileset::Tileset,
    util::{get_attrs, parse_tag},
    LayerTileRef, LayerType,
};

/// All Tiled map files will be parsed into this. Holds all the layers and tilesets.
#[derive(Debug, PartialEq, Clone)]
pub struct Map {
    /// The TMX format version this map was saved to.
    pub version: String,
    pub orientation: Orientation,
    /// Width of the map, in tiles.
    pub width: u32,
    /// Height of the map, in tiles.
    pub height: u32,
    /// Tile width, in pixels.
    pub tile_width: u32,
    /// Tile height, in pixels.
    pub tile_height: u32,
    /// The tilesets present in this map.
    pub tilesets: Vec<Tileset>,
    /// The layers present in this map.
    pub layers: Vec<Layer>,
    /// The custom properties of this map.
    pub properties: Properties,
    /// The background color of this map, if any.
    pub background_color: Option<Color>,
    pub infinite: bool,
}

impl Map {
    /// Parse a buffer hopefully containing the contents of a Tiled file and try to
    /// parse it. This augments `parse_file` with a custom reader: some engines
    /// (e.g. Amethyst) simply hand over a byte stream (and file location) for parsing,
    /// in which case this function may be required.
    ///
    /// The path is used for external dependencies such as tilesets or images, and may be skipped
    /// if the map is fully embedded (Doesn't refer to external files). If a map *does* refer to
    /// external files and a path is not given, the function will return [TiledError::SourceRequired].
    pub fn parse_reader<R: Read>(reader: R, path: Option<&Path>) -> Result<Self, TiledError> {
        let mut parser = EventReader::new(reader);
        loop {
            match parser.next().map_err(TiledError::XmlDecodingError)? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    if name.local_name == "map" {
                        return Self::parse_xml(&mut parser, attributes, path);
                    }
                }
                XmlEvent::EndDocument => {
                    return Err(TiledError::PrematureEnd(
                        "Document ended before map was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }

    /// Parse a file hopefully containing a Tiled map and try to parse it.  All external
    /// files will be loaded relative to the path given.
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, TiledError> {
        let file = File::open(path.as_ref())
            .map_err(|_| TiledError::Other(format!("Map file not found: {:?}", path.as_ref())))?;
        Self::parse_reader(file, Some(path.as_ref()))
    }

    /// Gets a tile from a [`Layer`] with the [`LayerType::TileLayer`] `layer_type` with the specified location.
    pub fn get_tile(&self, layer_index: usize, x: usize, y: usize) -> Option<LayerTileRef> {
        self.layers
            .get(layer_index)
            .and_then(|layer| match &layer.layer_type {
                LayerType::TileLayer(layer) => Some(layer),
                _ => None,
            })
            .and_then(|layer| layer.get_tile(x, y))
            .and_then(|layer_tile| LayerTileRef::from_gid(layer_tile, self))
    }
}

impl Map {
    pub(crate) fn get_tileset_for_gid(&self, gid: Gid) -> Option<&Tileset> {
        self.tilesets
            .iter()
            .rev()
            .find(|ts| ts.first_gid <= gid.0)
    }

    fn parse_xml<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        map_path: Option<&Path>,
    ) -> Result<Map, TiledError> {
        let ((c, infinite), (v, o, w, h, tw, th)) = get_attrs!(
            attrs,
            optionals: [
                ("backgroundcolor", colour, |v:String| v.parse().ok()),
                ("infinite", infinite, |v:String| Some(v == "1")),
            ],
            required: [
                ("version", version, |v| Some(v)),
                ("orientation", orientation, |v:String| v.parse().ok()),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
                ("tilewidth", tile_width, |v:String| v.parse().ok()),
                ("tileheight", tile_height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("map must have a version, width and height with correct types".to_string())
        );

        let infinite = infinite.unwrap_or(false);
        let source_path = map_path.and_then(|p| p.parent());

        let mut tilesets = Vec::new();
        let mut layers = Vec::new();
        let mut properties = HashMap::new();
        parse_tag!(parser, "map", {
            "tileset" => |attrs| {
                tilesets.push(Tileset::parse_xml(parser, attrs, source_path)?);
                Ok(())
            },
            "layer" => |attrs| {
                layers.push(Layer::new(parser, attrs, LayerTag::TileLayer, infinite, source_path)?);
                Ok(())
            },
            "imagelayer" => |attrs| {
                layers.push(Layer::new(parser, attrs, LayerTag::ImageLayer, infinite, source_path)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "objectgroup" => |attrs| {
                layers.push(Layer::new(parser, attrs, LayerTag::ObjectLayer, infinite, source_path)?);
                Ok(())
            },
        });
        Ok(Map {
            version: v,
            orientation: o,
            width: w,
            height: h,
            tile_width: tw,
            tile_height: th,
            tilesets,
            layers,
            properties,
            background_color: c,
            infinite,
        })
    }
}

/// Represents the way tiles are laid out in a map.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

impl FromStr for Orientation {
    type Err = ParseTileError;

    fn from_str(s: &str) -> Result<Orientation, ParseTileError> {
        match s {
            "orthogonal" => Ok(Orientation::Orthogonal),
            "isometric" => Ok(Orientation::Isometric),
            "staggered" => Ok(Orientation::Staggered),
            "hexagonal" => Ok(Orientation::Hexagonal),
            _ => Err(ParseTileError::OrientationError),
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Orientation::Orthogonal => write!(f, "orthogonal"),
            Orientation::Isometric => write!(f, "isometric"),
            Orientation::Staggered => write!(f, "staggered"),
            Orientation::Hexagonal => write!(f, "hexagonal"),
        }
    }
}

/// A Tiled global tile ID.
///
/// These are used to identify tiles in a map. Since the map may have more than one tileset, an
/// unique mapping is required to convert the tiles' local tileset ID to one which will work nicely
/// even if there is more than one tileset.
///
/// Tiled also treats GID 0 as empty space, which means that the first tileset in the map will have
/// a starting GID of 1.
///
/// See also: https://doc.mapeditor.org/en/latest/reference/global-tile-ids/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Gid(pub u32);

impl Gid {
    /// The GID representing an empty tile in the map.
    pub const EMPTY: Gid = Gid(0);
}
