use std::{collections::HashMap, fmt, fs::File, io::Read, path::Path, str::FromStr};

use xml::{attribute::OwnedAttribute, common::Position, reader::XmlEvent, EventReader};

use crate::{
    error::{ParseTileError, TiledError},
    layers::{LayerData, LayerTag},
    properties::{parse_properties, Color, Properties},
    tileset::Tileset,
    util::{get_attrs, parse_tag, XmlEventResult},
    EmbeddedParseResultType, Layer, ResourceCache, ResourcePath, TileLayerData,
};

#[derive(Debug, PartialEq, Clone)]
pub enum MapTilesetType {
    External { path: ResourcePath },
    Embedded { tileset: Tileset },
}

#[derive(Debug, PartialEq, Clone)]
pub struct MapTileset {
    pub(crate) first_gid: Gid,
    tileset_type: MapTilesetType,
}

impl MapTileset {
    /// Get a reference to the map tileset's type.
    pub fn tileset_type(&self) -> &MapTilesetType {
        &self.tileset_type
    }

    // HACK: Should this be in the interface?
    pub fn get_tileset<'ts>(&'ts self, cache: &'ts impl ResourceCache) -> Option<&'ts Tileset> {
        match &self.tileset_type {
            MapTilesetType::External { path } => cache.get_tileset(&path),
            MapTilesetType::Embedded { tileset } => Some(&tileset),
        }
    }
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
    /// The tilesets present on this map, ordered ascendingly by their first [`Gid`].
    tilesets: Vec<MapTileset>,
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
        tileset_cache: &mut impl ResourceCache,
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
                            tileset_cache,
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
        tileset_cache: &mut impl ResourceCache,
    ) -> Result<Self, TiledError> {
        let reader = File::open(path.as_ref())
            .map_err(|_| TiledError::Other(format!("Map file not found: {:?}", path.as_ref())))?;
        Self::parse_reader(reader, path.as_ref(), tileset_cache)
    }
}

impl Map {
    /// Get a reference to the map's tilesets.
    pub fn tilesets(&self) -> &[MapTileset] {
        self.tilesets.as_ref()
    }

    /// Get an iterator over all the layers in the map in ascending order of their layer index.
    pub fn layers(&self) -> LayerIter {
        LayerIter::new(self)
    }

    /// Returns the layer that has the specified index, if it exists.
    pub fn get_layer(&self, index: usize) -> Option<Layer> {
        self.layers.get(index).map(|data| Layer::new(self, data))
    }
}

/// An iterator that iterates over all the layers in a map, obtained via [`Map::layers`].
pub struct LayerIter<'map> {
    map: &'map Map,
    index: usize,
}

impl<'map> LayerIter<'map> {
    fn new(map: &'map Map) -> Self {
        Self { map, index: 0 }
    }
}

impl<'map> Iterator for LayerIter<'map> {
    type Item = Layer<'map>;

    fn next(&mut self) -> Option<Self::Item> {
        let layer_data = self.map.layers.get(self.index)?;
        self.index += 1;
        Some(Layer::new(self.map, layer_data))
    }
}

impl<'map> ExactSizeIterator for LayerIter<'map> {
    fn len(&self) -> usize {
        self.map.layers.len() - self.index
    }
}

impl Map {
    fn parse_xml(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        map_path: &Path,
        tileset_cache: &mut impl ResourceCache,
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

        // Since we can only parse sequentally, we cannot ensure tilesets are parsed before tile
        // layers. The issue here is that tile layers require tileset data in order to set
        // [`LayerTileData`] correctly. As such, we parse layer processing data into a vector as well as tilesets
        // and then, when all the tilesets are parsed, we actually construct each layer.
        let mut layers = Vec::new();
        let mut properties = HashMap::new();
        let mut tilesets = Vec::new();
        struct LayerProcessingData {
            attrs: Vec<OwnedAttribute>,
            events: Vec<XmlEventResult>,
            tag: LayerTag,
        }
        fn obtain_processing_data(
            attrs: Vec<OwnedAttribute>,
            events: &mut impl Iterator<Item = XmlEventResult>,
            tag: LayerTag,
            closing_tag: &str,
        ) -> Result<LayerProcessingData, TiledError> {
            let mut layer_events = Vec::new();
            for event in events {
                match event {
                    Ok(XmlEvent::EndElement { name, .. }) if name.local_name == closing_tag => {
                        return Ok(LayerProcessingData {
                            attrs,
                            events: layer_events,
                            tag,
                        });
                    }
                    _ => (),
                }
                layer_events.push(event);
            }
            Err(TiledError::PrematureEnd(
                "Couldn't obtain layer data".to_owned(),
            ))
        }

        parse_tag!(parser, "map", {
            "tileset" => |attrs| {
                let res = Tileset::parse_xml_in_map(parser, attrs, map_path)?;
                match res.result_type {
                    EmbeddedParseResultType::ExternalReference { tileset_path } => {
                        let file = File::open(&tileset_path).map_err(|err| TiledError::CouldNotOpenFile{path: tileset_path.clone(), err })?;
                        tileset_cache.get_or_try_insert_tileset_with(tileset_path.clone(), || Tileset::new_external(file, &tileset_path))?;
                        tilesets.push(MapTileset{first_gid: res.first_gid, tileset_type: MapTilesetType::External{ path: tileset_path.clone()}});
                    }
                    EmbeddedParseResultType::Embedded { tileset } => {
                        tilesets.push(MapTileset{first_gid: res.first_gid, tileset_type: MapTilesetType::Embedded {tileset}});
                    },
                };
                Ok(())
            },
            "layer" => |attrs| {
                layers.push(obtain_processing_data(attrs, parser, LayerTag::TileLayer, "layer")?);
                Ok(())
            },
            "imagelayer" => |attrs| {
                layers.push(obtain_processing_data(attrs, parser, LayerTag::ImageLayer, "imagelayer")?);
                Ok(())
            },
            "objectgroup" => |attrs| {
                layers.push(obtain_processing_data(attrs, parser, LayerTag::ObjectLayer, "objectgroup")?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        // Second pass: Process layers now that tilesets are all in
        let layers = layers
            .into_iter()
            .map(|data| {
                LayerData::new(
                    &mut data.events.into_iter(),
                    data.attrs,
                    data.tag,
                    infinite,
                    map_path,
                    &tilesets,
                )
            })
            .collect::<Result<Vec<_>, TiledError>>()?;

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
    #[allow(dead_code)]
    pub const EMPTY: Gid = Gid(0);
}
