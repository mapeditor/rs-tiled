use std::{collections::HashMap, fmt, fs::File, io::Read, path::Path, str::FromStr, sync::Arc};

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    error::TiledError,
    layers::{LayerData, LayerTag},
    properties::{parse_properties, Color, Properties},
    tileset::Tileset,
    util::{get_attrs, parse_tag, XmlEventResult},
    EmbeddedParseResultType, Layer, ResourceCache,
};

pub(crate) struct MapTilesetGid {
    pub first_gid: Gid,
    pub tileset: Arc<Tileset>,
}

/// All Tiled map files will be parsed into this. Holds all the layers and tilesets.
#[derive(PartialEq, Clone, Debug)]
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
    /// The tilesets present on this map.
    tilesets: Vec<Arc<Tileset>>,
    /// The layers present in this map.
    layers: Vec<LayerData>,
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
    /// The path is used for external dependencies such as tilesets or images. It is required.
    /// If the map if fully embedded and doesn't refer to external files, you may input an arbitrary path;
    /// the library won't read from the filesystem if it is not required to do so.
    ///
    /// The tileset cache is used to store and refer to any tilesets found along the way.
    pub fn parse_reader<R: Read>(
        reader: R,
        path: impl AsRef<Path>,
        cache: &mut impl ResourceCache,
    ) -> Result<Self, TiledError> {
        let mut parser = EventReader::new(reader);
        loop {
            match parser.next().map_err(TiledError::XmlDecodingError)? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    if name.local_name == "map" {
                        return Self::parse_xml(
                            &mut parser.into_iter(),
                            attributes,
                            path.as_ref(),
                            cache,
                        );
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
    ///
    /// The tileset cache is used to store and refer to any tilesets found along the way.
    pub fn parse_file(
        path: impl AsRef<Path>,
        cache: &mut impl ResourceCache,
    ) -> Result<Self, TiledError> {
        let reader = File::open(path.as_ref()).map_err(|err| TiledError::CouldNotOpenFile {
            path: path.as_ref().to_owned(),
            err,
        })?;
        Self::parse_reader(reader, path.as_ref(), cache)
    }
}

impl Map {
    /// Get a reference to the map's tilesets.
    pub fn tilesets(&self) -> &[Arc<Tileset>] {
        self.tilesets.as_ref()
    }

    /// Get an iterator over all the layers in the map in ascending order of their layer index.
    pub fn layers(&self) -> MapLayerIter {
        MapLayerIter::new(self)
    }

    /// Returns the layer that has the specified index, if it exists.
    pub fn get_layer(&self, index: usize) -> Option<Layer> {
        self.layers.get(index).map(|data| Layer::new(self, data))
    }
}

/// An iterator that iterates over all the layers in a map, obtained via [`Map::layers`].
pub struct MapLayerIter<'map> {
    map: &'map Map,
    index: usize,
}

impl<'map> MapLayerIter<'map> {
    fn new(map: &'map Map) -> Self {
        Self { map, index: 0 }
    }
}

impl<'map> Iterator for MapLayerIter<'map> {
    type Item = Layer<'map>;

    fn next(&mut self) -> Option<Self::Item> {
        let layer_data = self.map.layers.get(self.index)?;
        self.index += 1;
        Some(Layer::new(self.map, layer_data))
    }
}

impl<'map> ExactSizeIterator for MapLayerIter<'map> {
    fn len(&self) -> usize {
        self.map.layers.len() - self.index
    }
}

impl Map {
    fn parse_xml(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        map_path: &Path,
        cache: &mut impl ResourceCache,
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

        // We can only parse sequentally, but tilesets are guaranteed to appear before layers.
        // So we can pass in tileset data to layer construction without worrying about unfinished
        // data usage.
        let mut layers = Vec::new();
        let mut properties = HashMap::new();
        let mut tilesets = Vec::new();

        parse_tag!(parser, "map", {
            "tileset" => |attrs| {
                let res = Tileset::parse_xml_in_map(parser, attrs, map_path)?;
                match res.result_type {
                    EmbeddedParseResultType::ExternalReference { tileset_path } => {
                        let file = File::open(&tileset_path).map_err(|err| TiledError::CouldNotOpenFile{path: tileset_path.clone(), err })?;
                        let tileset = cache.get_or_try_insert_tileset_with(tileset_path.clone(), || Tileset::parse_reader(file, &tileset_path))?;
                        tilesets.push(MapTilesetGid{first_gid: res.first_gid, tileset});
                    }
                    EmbeddedParseResultType::Embedded { tileset } => {
                        tilesets.push(MapTilesetGid{first_gid: res.first_gid, tileset: Arc::new(tileset)});
                    },
                };
                Ok(())
            },
            "layer" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::TileLayer,
                    infinite,
                    map_path,
                    &tilesets,
                )?);
                Ok(())
            },
            "imagelayer" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::ImageLayer,
                    infinite,
                    map_path,
                    &tilesets,
                )?);
                Ok(())
            },
            "objectgroup" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::ObjectLayer,
                    infinite,
                    map_path,
                    &tilesets,
                )?);
                Ok(())
            },
            "group" => |attrs| {
                layers.push(LayerData::new(
                    parser,
                    attrs,
                    LayerTag::GroupLayer,
                    infinite,
                    map_path,
                    &tilesets,
                )?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        // We do not need first GIDs any more
        let tilesets = tilesets.into_iter().map(|ts| ts.tileset).collect();

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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "orthogonal" => Ok(Orientation::Orthogonal),
            "isometric" => Ok(Orientation::Isometric),
            "staggered" => Ok(Orientation::Staggered),
            "hexagonal" => Ok(Orientation::Hexagonal),
            _ => Err(()),
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
    #[allow(dead_code)]
    pub const EMPTY: Gid = Gid(0);
}
