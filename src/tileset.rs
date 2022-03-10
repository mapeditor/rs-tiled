use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use xml::attribute::OwnedAttribute;

use crate::error::{Error, Result};
use crate::image::Image;
use crate::properties::{parse_properties, Properties};
use crate::tile::TileData;
use crate::{util::*, Gid, Tile, TileId};

/// A collection of tiles for usage in maps and template objects.
///
/// Also see the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tileset).
#[derive(Debug, PartialEq, Clone)]
pub struct Tileset {
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
    pub tilecount: u32,
    /// The number of tile columns in the tileset. Editable for image collection tilesets, otherwise
    /// calculated using [image](Self::image) width, [tile width](Self::tile_width),
    /// [spacing](Self::spacing) and [margin](Self::margin).
    pub columns: u32,

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

    /// The custom properties of the tileset.
    pub properties: Properties,
}

pub(crate) enum EmbeddedParseResultType {
    ExternalReference { tileset_path: PathBuf },
    Embedded { tileset: Tileset },
}

pub(crate) struct EmbeddedParseResult {
    pub first_gid: Gid,
    pub result_type: EmbeddedParseResultType,
}

/// Internal structure for holding mid-parse information.
struct TilesetProperties {
    spacing: Option<u32>,
    margin: Option<u32>,
    tilecount: u32,
    columns: Option<u32>,
    name: String,
    tile_width: u32,
    tile_height: u32,
    /// The root all non-absolute paths contained within the tileset are relative to.
    root_path: PathBuf,
}

impl Tileset {
    /// Parses a tileset out of a reader hopefully containing the contents of a Tiled tileset.
    /// Uses the `path` parameter as the root for any relative paths found in the tileset.
    ///
    /// ## Example
    /// ```
    /// use std::fs::File;
    /// use std::path::PathBuf;
    /// use std::io::BufReader;
    /// use tiled::Tileset;
    ///
    /// let path = "assets/tilesheet.tsx";
    /// let reader = BufReader::new(File::open(path).unwrap());
    /// let tileset = Tileset::parse_reader(reader, path).unwrap();
    ///
    /// assert_eq!(tileset.image.unwrap().source, PathBuf::from("assets/tilesheet.png"));
    /// ```
    #[deprecated(since = "0.10.1", note = "Use `Loader::parse_tsx_tileset` instead")]
    pub fn parse_reader<R: Read>(reader: R, path: impl AsRef<Path>) -> Result<Self> {
        crate::parse::xml::parse_tileset(reader, path.as_ref())
    }

    /// Gets the tile with the specified ID from the tileset.
    #[inline]
    pub fn get_tile(&self, id: TileId) -> Option<Tile> {
        self.tiles.get(&id).map(|data| Tile::new(self, data))
    }

    /// Iterates through the tiles from this tileset.
    #[inline]
    pub fn tiles(&self) -> impl ExactSizeIterator<Item = (TileId, Tile)> {
        self.tiles
            .iter()
            .map(move |(id, data)| (*id, Tile::new(self, data)))
    }
}

impl Tileset {
    pub(crate) fn parse_xml_in_map(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        map_path: &Path,
    ) -> Result<EmbeddedParseResult> {
        Tileset::parse_xml_embedded(parser, &attrs, map_path).or_else(|err| {
            if matches!(err, Error::MalformedAttributes(_)) {
                Tileset::parse_xml_reference(&attrs, map_path)
            } else {
                Err(err)
            }
        })
    }

    fn parse_xml_embedded(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: &Vec<OwnedAttribute>,
        map_path: &Path,
    ) -> Result<EmbeddedParseResult> {
        let ((spacing, margin, columns, name), (tilecount, first_gid, tile_width, tile_height)) = get_attrs!(
           attrs,
           optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
                ("columns", columns, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
            ],
           required: [
                ("tilecount", tilecount, |v:String| v.parse().ok()),
                ("firstgid", first_gid, |v:String| v.parse().ok().map(|n| Gid(n))),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let root_path = map_path.parent().ok_or(Error::PathIsNotFile)?.to_owned();

        Self::finish_parsing_xml(
            parser,
            TilesetProperties {
                spacing,
                margin,
                name: name.unwrap_or_default(),
                root_path,
                columns,
                tilecount,
                tile_height,
                tile_width,
            },
        )
        .map(|tileset| EmbeddedParseResult {
            first_gid,
            result_type: EmbeddedParseResultType::Embedded { tileset },
        })
    }

    fn parse_xml_reference(
        attrs: &Vec<OwnedAttribute>,
        map_path: &Path,
    ) -> Result<EmbeddedParseResult> {
        let (first_gid, source) = get_attrs!(
            attrs,
            required: [
                ("firstgid", first_gid, |v:String| v.parse().ok().map(|n| Gid(n))),
                ("source", name, |v| Some(v)),
            ],
            Error::MalformedAttributes("Tileset reference must have a firstgid and source with correct types".to_string())
        );

        let tileset_path = map_path.parent().ok_or(Error::PathIsNotFile)?.join(source);

        Ok(EmbeddedParseResult {
            first_gid,
            result_type: EmbeddedParseResultType::ExternalReference { tileset_path },
        })
    }

    pub(crate) fn parse_external_tileset(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: &Vec<OwnedAttribute>,
        path: &Path,
    ) -> Result<Tileset> {
        let ((spacing, margin, columns, name), (tilecount, tile_width, tile_height)) = get_attrs!(
            attrs,
            optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
                ("columns", columns, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
            ],
            required: [
                ("tilecount", tilecount, |v:String| v.parse().ok()),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("tileset must have a name, tile width and height with correct types".to_string())
        );

        let root_path = path.parent().ok_or(Error::PathIsNotFile)?.to_owned();

        Self::finish_parsing_xml(
            parser,
            TilesetProperties {
                spacing,
                margin,
                name: name.unwrap_or_default(),
                root_path,
                columns,
                tilecount,
                tile_height,
                tile_width,
            },
        )
    }

    fn finish_parsing_xml(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        prop: TilesetProperties,
    ) -> Result<Tileset> {
        let mut image = Option::None;
        let mut tiles = HashMap::with_capacity(prop.tilecount as usize);
        let mut properties = HashMap::new();

        parse_tag!(parser, "tileset", {
            "image" => |attrs| {
                image = Some(Image::new(parser, attrs, &prop.root_path)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
            "tile" => |attrs| {
                let (id, tile) = TileData::new(parser, attrs, &prop.root_path)?;
                tiles.insert(id, tile);
                Ok(())
            },
        });

        // A tileset is considered an image collection tileset if there is no image attribute (because its tiles do).
        let is_image_collection_tileset = image.is_none();

        if !is_image_collection_tileset {
            for tile_id in 0..prop.tilecount {
                tiles.entry(tile_id).or_default();
            }
        }

        let margin = prop.margin.unwrap_or(0);
        let spacing = prop.spacing.unwrap_or(0);
        let columns = prop
            .columns
            .map(Ok)
            .unwrap_or_else(|| Self::calculate_columns(&image, prop.tile_width, margin, spacing))?;

        Ok(Tileset {
            name: prop.name,
            tile_width: prop.tile_width,
            tile_height: prop.tile_height,
            spacing,
            margin,
            columns,
            tilecount: prop.tilecount,
            image,
            tiles,
            properties,
        })
    }

    fn calculate_columns(
        image: &Option<Image>,
        tile_width: u32,
        margin: u32,
        spacing: u32,
    ) -> Result<u32> {
        image
            .as_ref()
            .ok_or(Error::MalformedAttributes(
                "No <image> nor columns attribute in <tileset>".to_string(),
            ))
            .and_then(|image| Ok((image.width as u32 - margin + spacing) / (tile_width + spacing)))
    }
}
