use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::image::Image;
use crate::properties::{Properties, parse_properties};
use crate::tile::TileData;
use crate::{
    Gid, InvalidTilesetError, ResourceCache, ResourceReader, Tile, TileId,
    util::{get_attrs, parse_tag},
};

mod wangset;
pub use wangset::*;

/// A collection of tiles for usage in maps and template objects.
///
/// Also see the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tileset).
#[derive(Debug, PartialEq, Clone)]
pub struct Tileset {
    /// The path first used in a [`ResourceReader`] to load this tileset.
    ///
    /// For embedded tilesets, this path will be the same as the template or map's source.
    pub source: PathBuf,
    /// The name of the tileset, set by the user.
    pub name: String,
    /// The (maximum) width in pixels of the tiles in this tileset. Irrelevant for [image collection]
    /// tilesets.
    ///
    /// [image collection]: Self::image
    pub tile_width: u32,
    /// The (maximum) height in pixels of the tiles in this tileset. Irrelevant for [image collection]
    /// tilesets.
    ///
    /// [image collection]: Self::image
    pub tile_height: u32,
    /// The spacing in pixels between the tiles in this tileset (applies to the tileset image).
    /// Irrelevant for image collection tilesets.
    pub spacing: u32,
    /// The margin around the tiles in this tileset (applies to the tileset image).
    /// Irrelevant for image collection tilesets.
    pub margin: u32,
    /// The number of tiles in this tileset. Note that tile IDs don't always have a connection with
    /// the tile count, and as such there may be tiles with an ID bigger than the tile count.
    /// When omitted, this is inferred from the sheet dimensions or the number of collection tiles.
    pub tilecount: u32,
    /// The number of tile columns in the tileset. Editable for image collection tilesets, otherwise
    /// calculated using [image](Self::image) width, [tile width](Self::tile_width),
    /// [spacing](Self::spacing) and [margin](Self::margin). Defaults to zero for image collections.
    pub columns: u32,
    /// The x-offset to be used when drawing tiles of this tileset.
    pub offset_x: i32,
    /// The y-offset to be used when drawing tiles of this tileset.
    pub offset_y: i32,
    /// The size to use when rendering tiles of this tileset on a tile layer.
    pub tile_render_size: TileRenderSize,
    /// The fill mode to use when rendering tiles of this tileset, relevant when the tiles are
    /// not rendered at their native size.
    pub fill_mode: FillMode,
    /// The alignment to use for tile objects referring to tiles of this tileset.
    pub object_alignment: ObjectAlignment,
    /// The transformations that can be applied to the tiles of this tileset, for example when
    /// used by Wang sets.
    pub transformations: Transformations,

    /// A tileset can either:
    /// * have a single spritesheet `image` in `tileset` ("regular" tileset);
    /// * have zero images in `tileset` and one `image` per `tile` ("image collection" tileset).
    ///
    /// --------
    /// - Source: [tiled issue #2117](https://github.com/mapeditor/tiled/issues/2117)
    /// - Source: [`columns` documentation](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tileset)
    pub image: Option<Image>,

    /// All the tiles present in this tileset, indexed by their local IDs.
    tiles: HashMap<TileId, TileData>,

    /// All the wangsets present in this tileset.
    pub wang_sets: Vec<WangSet>,

    /// The custom properties of the tileset.
    pub properties: Properties,

    /// The custom tileset type, arbitrarily set by the user.
    pub user_type: String,
}

/// The transformations that can be applied to the tiles of a tileset, for example when used by
/// Wang sets.
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub struct Transformations {
    /// Whether the tiles in this tileset can be flipped horizontally.
    pub hflip: bool,
    /// Whether the tiles in this tileset can be flipped vertically.
    pub vflip: bool,
    /// Whether the tiles in this tileset can be rotated in 90-degree increments.
    pub rotate: bool,
    /// Whether untransformed tiles remain preferred, otherwise transformed tiles are used to
    /// produce more variations.
    pub prefer_untransformed: bool,
}

/// The size to use when rendering a tileset's tiles on a tile layer.
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum TileRenderSize {
    /// The tile is drawn at its own size, positioned so that its bottom left corner aligns with
    /// the bottom left corner of its grid cell.
    #[default]
    Tile,
    /// The tile is drawn at the size of the map's grid cell.
    Grid,
}

#[derive(Debug)]
/// An error arising from trying to parse a [`TileRenderSize`] that is not valid.
pub struct TileRenderSizeParseError {
    /// The invalid string found.
    pub str_found: String,
}

impl fmt::Display for TileRenderSizeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "failed to parse tile render size, valid options are `tile` and `grid` \
        but got `{}` instead",
            self.str_found
        ))
    }
}

impl FromStr for TileRenderSize {
    type Err = TileRenderSizeParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "tile" => Ok(TileRenderSize::Tile),
            "grid" => Ok(TileRenderSize::Grid),
            _ => Err(TileRenderSizeParseError {
                str_found: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for TileRenderSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileRenderSize::Tile => write!(f, "tile"),
            TileRenderSize::Grid => write!(f, "grid"),
        }
    }
}

/// The fill mode to use when rendering a tileset's tiles at a size differing from their native
/// size.
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum FillMode {
    /// The tile image is stretched to fill the target rectangle.
    #[default]
    Stretch,
    /// The tile image is scaled to fit the target rectangle while preserving its aspect ratio.
    PreserveAspectFit,
}

#[derive(Debug)]
/// An error arising from trying to parse a [`FillMode`] that is not valid.
pub struct FillModeParseError {
    /// The invalid string found.
    pub str_found: String,
}

impl fmt::Display for FillModeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "failed to parse fill mode, valid options are `stretch` and `preserve-aspect-fit` \
        but got `{}` instead",
            self.str_found
        ))
    }
}

impl FromStr for FillMode {
    type Err = FillModeParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "stretch" => Ok(FillMode::Stretch),
            "preserve-aspect-fit" => Ok(FillMode::PreserveAspectFit),
            _ => Err(FillModeParseError {
                str_found: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for FillMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FillMode::Stretch => write!(f, "stretch"),
            FillMode::PreserveAspectFit => write!(f, "preserve-aspect-fit"),
        }
    }
}

/// The alignment to use for tile objects referring to a tileset's tiles.
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
#[allow(missing_docs)]
pub enum ObjectAlignment {
    /// Tile objects use the alignment that older Tiled versions used: [`BottomLeft`] in
    /// orthogonal maps and [`Bottom`] in isometric maps.
    ///
    /// [`BottomLeft`]: ObjectAlignment::BottomLeft
    /// [`Bottom`]: ObjectAlignment::Bottom
    #[default]
    Unspecified,
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug)]
/// An error arising from trying to parse an [`ObjectAlignment`] that is not valid.
pub struct ObjectAlignmentParseError {
    /// The invalid string found.
    pub str_found: String,
}

impl fmt::Display for ObjectAlignmentParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "failed to parse object alignment, valid options are `unspecified`, `topleft`, \
        `top`, `topright`, `left`, `center`, `right`, `bottomleft`, `bottom` and `bottomright` \
        but got `{}` instead",
            self.str_found
        ))
    }
}

impl FromStr for ObjectAlignment {
    type Err = ObjectAlignmentParseError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "unspecified" => Ok(ObjectAlignment::Unspecified),
            "topleft" => Ok(ObjectAlignment::TopLeft),
            "top" => Ok(ObjectAlignment::Top),
            "topright" => Ok(ObjectAlignment::TopRight),
            "left" => Ok(ObjectAlignment::Left),
            "center" => Ok(ObjectAlignment::Center),
            "right" => Ok(ObjectAlignment::Right),
            "bottomleft" => Ok(ObjectAlignment::BottomLeft),
            "bottom" => Ok(ObjectAlignment::Bottom),
            "bottomright" => Ok(ObjectAlignment::BottomRight),
            _ => Err(ObjectAlignmentParseError {
                str_found: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for ObjectAlignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectAlignment::Unspecified => write!(f, "unspecified"),
            ObjectAlignment::TopLeft => write!(f, "topleft"),
            ObjectAlignment::Top => write!(f, "top"),
            ObjectAlignment::TopRight => write!(f, "topright"),
            ObjectAlignment::Left => write!(f, "left"),
            ObjectAlignment::Center => write!(f, "center"),
            ObjectAlignment::Right => write!(f, "right"),
            ObjectAlignment::BottomLeft => write!(f, "bottomleft"),
            ObjectAlignment::Bottom => write!(f, "bottom"),
            ObjectAlignment::BottomRight => write!(f, "bottomright"),
        }
    }
}

pub(crate) enum EmbeddedParseResultType {
    ExternalReference { tileset_path: PathBuf },
    Embedded { tileset: Box<Tileset> },
}

pub(crate) struct EmbeddedParseResult {
    pub first_gid: Gid,
    pub result_type: EmbeddedParseResultType,
}

/// Internal structure for holding mid-parse information.
struct TilesetProperties {
    spacing: Option<u32>,
    margin: Option<u32>,
    tilecount: Option<u32>,
    columns: Option<u32>,
    name: String,
    user_type: String,
    tile_width: u32,
    tile_height: u32,
    tile_render_size: Option<TileRenderSize>,
    fill_mode: Option<FillMode>,
    object_alignment: Option<ObjectAlignment>,
    /// The root all non-absolute paths contained within the tileset are relative to.
    root_path: PathBuf,
}

impl Tileset {
    /// Gets the tile with the specified ID from the tileset.
    #[inline]
    pub fn get_tile(&self, id: TileId) -> Option<Tile<'_>> {
        self.tiles.get(&id).map(|data| Tile::new(self, data))
    }

    /// Iterates through the tiles from this tileset.
    #[inline]
    pub fn tiles(&self) -> impl ExactSizeIterator<Item = (TileId, Tile<'_>)> {
        self.tiles
            .iter()
            .map(move |(id, data)| (*id, Tile::new(self, data)))
    }
}

impl Tileset {
    pub(crate) fn parse_xml_in_map<R: std::io::BufRead>(
        elem: crate::util::XmlElement<'_, R>,
        path: &Path, // Template or Map file
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<EmbeddedParseResult> {
        let (first_gid, source) = get_attrs!(
            for v in (elem.attrs) {
                "firstgid" => first_gid ?= v.parse::<u32>().map(Gid),
                Some("source") => source = v.to_string(),
            }
            (first_gid, source)
        );

        let result_type = if let Some(source) = source {
            let tileset_path = path.parent().ok_or(Error::PathIsNotFile)?.join(source);
            parse_tag!(elem, {});
            EmbeddedParseResultType::ExternalReference { tileset_path }
        } else {
            let tileset = Self::parse_xml(elem, path, reader, cache)?;
            EmbeddedParseResultType::Embedded {
                tileset: Box::new(tileset),
            }
        };
        Ok(EmbeddedParseResult {
            first_gid,
            result_type,
        })
    }

    pub(crate) fn parse_xml<R: std::io::BufRead>(
        elem: crate::util::XmlElement<'_, R>,
        path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Tileset> {
        let (
            (spacing, margin, columns, name, user_type, user_class),
            (tile_render_size, fill_mode, object_alignment),
            (tilecount, tile_width, tile_height),
        ) = get_attrs!(
            for v in (elem.attrs) {
                Some("spacing") => spacing ?= v.parse(),
                Some("margin") => margin ?= v.parse(),
                Some("columns") => columns ?= v.parse(),
                Some("name") => name = v.to_string(),
                Some("type") => user_type ?= v.parse(),
                Some("class") => user_class ?= v.parse(),
                Some("tilerendersize") => tile_render_size ?= v.parse::<TileRenderSize>(),
                Some("fillmode") => fill_mode ?= v.parse::<FillMode>(),
                Some("objectalignment") => object_alignment ?= v.parse::<ObjectAlignment>(),

                Some("tilecount") => tilecount ?= v.parse::<u32>(),
                "tilewidth" => tile_width ?= v.parse::<u32>(),
                "tileheight" => tile_height ?= v.parse::<u32>(),
            }
            ((spacing, margin, columns, name, user_type, user_class), (tile_render_size, fill_mode, object_alignment), (tilecount, tile_width, tile_height))
        );

        let root_path = path.parent().ok_or(Error::PathIsNotFile)?.to_owned();

        Self::finish_parsing_xml(
            elem,
            path.to_owned(),
            TilesetProperties {
                spacing,
                margin,
                name: name.unwrap_or_default(),
                user_type: user_type.or(user_class).unwrap_or_default(),
                root_path,
                columns,
                tilecount,
                tile_height,
                tile_width,
                tile_render_size,
                fill_mode,
                object_alignment,
            },
            reader,
            cache,
        )
    }

    fn finish_parsing_xml<R: std::io::BufRead>(
        elem: crate::util::XmlElement<'_, R>,
        container_path: PathBuf,
        prop: TilesetProperties,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<Tileset> {
        let mut image = Option::None;
        let mut tiles = HashMap::with_capacity(prop.tilecount.unwrap_or_default() as usize);
        let mut properties = HashMap::new();
        let mut wang_sets = Vec::new();
        let mut offset = (0i32, 0i32);
        let mut transformations = Transformations::default();

        parse_tag!(elem, {
            "image" => |elem| {
                image = Some(Image::new(elem, &prop.root_path)?);
                Ok(())
            },
            "tileoffset" => |elem| {
                offset = parse_tileoffset(elem)?;
                Ok(())
            },
            "transformations" => |elem| {
                transformations = parse_transformations(elem)?;
                Ok(())
            },
            "properties" => |elem| {
                properties = parse_properties(elem)?;
                Ok(())
            },
            "tile" => |elem| {
                let (id, tile) = TileData::new(elem, &prop.root_path, reader, cache)?;
                tiles.insert(id, tile);
                Ok(())
            },
            "wangsets" => |elem: crate::util::XmlElement<'_, R>| {
                parse_tag!(elem, {
                    "wangset" => |elem| {
                        let set = WangSet::new(elem)?;
                        wang_sets.push(set);
                        Ok(())
                    },
                });
                Ok(())
            },
        });

        let margin = prop.margin.unwrap_or(0);
        let spacing = prop.spacing.unwrap_or(0);
        let too_many_tiles = || Error::MalformedAttributes("Tile count exceeds u32".into());
        let (columns, tilecount) = if let Some(image) = &image {
            if prop.tile_width == 0 || prop.tile_height == 0 {
                return Err(Error::InvalidTileset(
                    InvalidTilesetError::InvalidTileDimensions,
                ));
            }

            // Use the same leading margin and inter-tile spacing on either axis.
            let tiles_along = |pixels: i32, tile: u32| {
                let available = i64::from(pixels) - i64::from(margin) + i64::from(spacing);
                (available.max(0) / (i64::from(tile) + i64::from(spacing))) as u32
            };
            let image_columns = tiles_along(image.width, prop.tile_width);
            let tilecount = match prop.tilecount {
                Some(count) => count,
                None => image_columns
                    .checked_mul(tiles_along(image.height, prop.tile_height))
                    .ok_or_else(too_many_tiles)?,
            };
            for tile_id in 0..tilecount {
                tiles.entry(tile_id).or_default();
            }
            (prop.columns.unwrap_or(image_columns), tilecount)
        } else {
            let count = match prop.tilecount {
                Some(count) => count,
                None => u32::try_from(tiles.len()).map_err(|_| too_many_tiles())?,
            };
            (prop.columns.unwrap_or_default(), count)
        };

        Ok(Tileset {
            source: container_path,
            name: prop.name,
            user_type: prop.user_type,
            tile_width: prop.tile_width,
            tile_height: prop.tile_height,
            spacing,
            margin,
            columns,
            offset_x: offset.0,
            offset_y: offset.1,
            tile_render_size: prop.tile_render_size.unwrap_or_default(),
            fill_mode: prop.fill_mode.unwrap_or_default(),
            object_alignment: prop.object_alignment.unwrap_or_default(),
            transformations,
            tilecount,
            image,
            tiles,
            wang_sets,
            properties,
        })
    }
}

/// Parse the optional <transformations hflip=... vflip=... rotate=... preferuntransformed=.../> tag.
fn parse_transformations<R: std::io::BufRead>(
    elem: crate::util::XmlElement<'_, R>,
) -> Result<Transformations> {
    let (hflip, vflip, rotate, prefer_untransformed) = get_attrs!(
        for v in (elem.attrs) {
            Some("hflip") => hflip = v == "1",
            Some("vflip") => vflip = v == "1",
            Some("rotate") => rotate = v == "1",
            Some("preferuntransformed") => prefer_untransformed = v == "1",
        }
        (hflip, vflip, rotate, prefer_untransformed)
    );
    parse_tag!(elem, {});
    Ok(Transformations {
        hflip: hflip.unwrap_or(false),
        vflip: vflip.unwrap_or(false),
        rotate: rotate.unwrap_or(false),
        prefer_untransformed: prefer_untransformed.unwrap_or(false),
    })
}

/// Parse the optional <tileoffset x=... y=.../> tag.
fn parse_tileoffset<R: std::io::BufRead>(
    elem: crate::util::XmlElement<'_, R>,
) -> Result<(i32, i32)> {
    let offset = get_attrs!(
        for v in (elem.attrs) {
            "x" => offset_x ?= v.parse::<i32>(),
            "y" => offset_y ?= v.parse::<i32>(),
        }
        (offset_x, offset_y)
    );
    parse_tag!(elem, {});
    Ok(offset)
}
