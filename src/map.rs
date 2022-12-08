//! Structures related to Tiled maps.

use std::{collections::HashMap, fmt, fs::File, io::Read, path::Path, str::FromStr, sync::Arc};

use xml::attribute::OwnedAttribute;

use crate::{
    error::{Error, Result},
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
    version: String,
    /// The way tiles are laid out in the map.
    pub orientation: Orientation,
    /// Width of the map, in tiles.
    ///
    /// ## Note
    /// There is no guarantee that this value will be the same as the width from its tile layers.
    pub width: u32,
    /// Height of the map, in tiles.
    ///
    /// ## Note
    /// There is no guarantee that this value will be the same as the height from its tile layers.
    pub height: u32,
    /// Tile width, in pixels.
    ///
    /// ## Note
    /// This value along with [`Self::tile_height`] determine the general size of the map, and
    /// individual tiles may have different sizes. As such, there is no guarantee that this value
    /// will be the same as the one from the tilesets the map is using.
    pub tile_width: u32,
    /// Tile height, in pixels.
    ///
    /// ## Note
    /// This value along with [`Self::tile_width`] determine the general size of the map, and
    /// individual tiles may have different sizes. As such, there is no guarantee that this value
    /// will be the same as the one from the tilesets the map is using.
    pub tile_height: u32,
    /// The tilesets present on this map.
    tilesets: Vec<Arc<Tileset>>,
    /// The layers present in this map.
    layers: Vec<LayerData>,
    /// The custom properties of this map.
    pub properties: Properties,
    /// The background color of this map, if any.
    pub background_color: Option<Color>,
    infinite: bool,
    /// The type of the map, which is arbitrary and set by the user.
    pub map_type: String,
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
    #[deprecated(since = "0.10.1", note = "Use `Loader::load_tmx_map_from` instead")]
    pub fn parse_reader<R: Read>(
        reader: R,
        path: impl AsRef<Path>,
        cache: &mut impl ResourceCache,
    ) -> Result<Self> {
        crate::parse::xml::parse_map(reader, path.as_ref(), cache)
    }

    /// Parse a file hopefully containing a Tiled map and try to parse it.  All external
    /// files will be loaded relative to the path given.
    ///
    /// The tileset cache is used to store and refer to any tilesets found along the way.
    #[deprecated(since = "0.10.1", note = "Use `Loader::load_tmx_map` instead")]
    pub fn parse_file(path: impl AsRef<Path>, cache: &mut impl ResourceCache) -> Result<Self> {
        let reader = File::open(path.as_ref()).map_err(|err| Error::CouldNotOpenFile {
            path: path.as_ref().to_owned(),
            err,
        })?;
        crate::parse::xml::parse_map(reader, path.as_ref(), cache)
    }

    /// The TMX format version this map was saved to. Equivalent to the map file's `version`
    /// attribute.
    pub fn version(&self) -> &str {
        self.version.as_ref()
    }

    /// Whether this map is infinite. An infinite map has no fixed size and can grow in all
    /// directions. Its layer data is stored in chunks. This value determines whether the map's
    /// tile layers are [`FiniteTileLayer`](crate::FiniteTileLayer)s or [`crate::InfiniteTileLayer`](crate::InfiniteTileLayer)s.
    pub fn infinite(&self) -> bool {
        self.infinite
    }
}

impl Map {
    /// Get a reference to the map's tilesets.
    #[inline]
    pub fn tilesets(&self) -> &[Arc<Tileset>] {
        self.tilesets.as_ref()
    }

    /// Get an iterator over all the layers in the map in ascending order of their layer index.
    ///
    /// ## Example
    /// ```
    /// # use tiled::Loader;
    /// #
    /// # fn main() {
    /// # struct Renderer;
    /// # impl Renderer {
    /// #     fn render(&self, _: tiled::TileLayer) {}
    /// # }
    /// # let my_renderer = Renderer;
    /// # let map = Loader::new()
    /// #     .load_tmx_map("assets/tiled_group_layers.tmx")
    /// #     .unwrap();
    /// #
    /// let tile_layers = map.layers().filter_map(|layer| match layer.layer_type() {
    ///     tiled::LayerType::TileLayer(layer) => Some(layer),
    ///     _ => None,
    /// });
    ///
    /// for layer in tile_layers {
    ///     my_renderer.render(layer);
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn layers(&self) -> impl ExactSizeIterator<Item = Layer> {
        self.layers.iter().map(move |layer| Layer::new(self, layer))
    }

    /// Returns the layer that has the specified index, if it exists.
    pub fn get_layer(&self, index: usize) -> Option<Layer> {
        self.layers.get(index).map(|data| Layer::new(self, data))
    }
}

impl Map {
    pub(crate) fn parse_xml(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        map_path: &Path,
        cache: &mut impl ResourceCache,
    ) -> Result<Map> {
        let ((c, infinite, user_type), (v, o, w, h, tw, th)) = get_attrs!(
            for v in attrs {
                Some("backgroundcolor") => colour ?= v.parse(),
                Some("infinite") => infinite = v == "1",
                Some("class") => user_type ?= v.parse(),
                "version" => version = v,
                "orientation" => orientation ?= v.parse::<Orientation>(),
                "width" => width ?= v.parse::<u32>(),
                "height" => height ?= v.parse::<u32>(),
                "tilewidth" => tile_width ?= v.parse::<u32>(),
                "tileheight" => tile_height ?= v.parse::<u32>(),
            }
            ((colour, infinite, user_type), (version, orientation, width, height, tile_width, tile_height))
        );

        let infinite = infinite.unwrap_or(false);
        let map_type = user_type.unwrap_or_default();

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
                        let file = File::open(&tileset_path).map_err(|err| Error::CouldNotOpenFile{path: tileset_path.clone(), err })?;
                        let tileset = cache.get_or_try_insert_tileset_with(tileset_path.clone(), || crate::parse::xml::parse_tileset(file, &tileset_path))?;
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
            map_type,
        })
    }
}

/// Represents the way tiles are laid out in a map.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(missing_docs)]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

impl FromStr for Orientation {
    // TODO(0.11): Change error type to OrientationParseErr or similar
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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
