use std::{collections::HashMap, path::Path, sync::Arc};

use xml::attribute::OwnedAttribute;

use crate::{
    error::{Error, Result},
    properties::{parse_properties, Properties},
    template::Template,
    util::{get_attrs, map_wrapper, parse_tag, XmlEventResult},
    Gid, MapTilesetGid, ResourceCache, ResourceReader, Tile, TileId, Tileset,
};

/// The location of the tileset this tile is in
///
/// Tilesets can be contained within either a map or a template.
#[derive(Clone, Debug, PartialEq)]
pub enum TilesetLocation {
    /// Index into the Map's tileset list, guaranteed to be a valid index of the map tileset container.
    Map(usize),
    /// Arc of the tileset itself if and only if this location is from a template.
    Template(Arc<Tileset>),
}

/// Stores the internal tile gid about a layer tile, along with how it is flipped.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectTileData {
    /// A valid TilesetLocation that points to a tileset that **may or may not contain** this tile.
    tileset_location: TilesetLocation,
    /// The local ID of the tile in the tileset it's in.
    id: TileId,
    /// Whether this tile is flipped on its Y axis (horizontally).
    pub flip_h: bool,
    /// Whether this tile is flipped on its X axis (vertically).
    pub flip_v: bool,
    /// Whether this tile is flipped diagonally.
    pub flip_d: bool,
}

impl ObjectTileData {
    /// Get the layer tile's local id within its parent tileset.
    #[inline]
    pub fn id(&self) -> TileId {
        self.id
    }

    /// Get a reference to the object tile data's tileset location, which points to a tileset that
    /// **may or may not contain** this tile.
    #[inline]
    pub fn tileset_location(&self) -> &TilesetLocation {
        &self.tileset_location
    }

    const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
    const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
    const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;
    const ALL_FLIP_FLAGS: u32 = Self::FLIPPED_HORIZONTALLY_FLAG
        | Self::FLIPPED_VERTICALLY_FLAG
        | Self::FLIPPED_DIAGONALLY_FLAG;

    /// Creates a new [`ObjectTileData`] from a [`Gid`] plus its flipping bits.
    pub(crate) fn from_bits(
        bits: u32,
        tilesets: &[MapTilesetGid],
        for_tileset: Option<Arc<Tileset>>,
    ) -> Option<Self> {
        let flags = bits & Self::ALL_FLIP_FLAGS;
        let gid = Gid(bits & !Self::ALL_FLIP_FLAGS);
        let flip_d = flags & Self::FLIPPED_DIAGONALLY_FLAG == Self::FLIPPED_DIAGONALLY_FLAG; // Swap x and y axis (anti-diagonally) [flips over y = -x line]
        let flip_h = flags & Self::FLIPPED_HORIZONTALLY_FLAG == Self::FLIPPED_HORIZONTALLY_FLAG; // Flip tile over y axis
        let flip_v = flags & Self::FLIPPED_VERTICALLY_FLAG == Self::FLIPPED_VERTICALLY_FLAG; // Flip tile over x axis

        if gid == Gid::EMPTY {
            None
        } else {
            let (tileset_location, id) = match for_tileset {
                Some(tileset) => (TilesetLocation::Template(tileset), gid.0 - 1),
                None => {
                    let (tileset_index, tileset) = crate::util::get_tileset_for_gid(tilesets, gid)?;
                    let id = gid.0 - tileset.first_gid.0;
                    (TilesetLocation::Map(tileset_index), id)
                }
            };

            Some(Self {
                tileset_location,
                id,
                flip_h,
                flip_v,
                flip_d,
            })
        }
    }
}

map_wrapper!(
    #[doc = "An instance of a [`Tile`] present in an [`Object`]."]
    ObjectTile => ObjectTileData
);

impl<'map> ObjectTile<'map> {
    /// Get a reference to the object tile's referenced tile, if it exists.
    #[inline]
    pub fn get_tile(&self) -> Option<Tile<'map>> {
        self.get_tileset().get_tile(self.data.id)
    }
    /// Get a reference to the object tile's referenced tileset.
    #[inline]
    pub fn get_tileset(&self) -> &'map Tileset {
        match &self.data.tileset_location {
            // SAFETY: `tileset_index` is guaranteed to be valid
            TilesetLocation::Map(n) => &self.map.tilesets()[*n],
            TilesetLocation::Template(t) => t,
        }
    }
}

/// A structure describing an [`Object`]'s shape.
///
/// Also see the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tmx-object).
#[derive(Debug, PartialEq, Clone)]
#[allow(missing_docs)]
pub enum ObjectShape {
    Rect { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Polyline { points: Vec<(f32, f32)> },
    Polygon { points: Vec<(f32, f32)> },
    Point(f32, f32),
}

/// Raw data belonging to an object. Used internally and for tile collisions.
///
/// Also see the [TMX docs](https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#tmx-object).
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectData {
    id: u32,
    tile: Option<ObjectTileData>,
    /// The name of the object, which is arbitrary and set by the user.
    pub name: String,
    /// The type of the object, which is arbitrary and set by the user.
    pub obj_type: String,
    /// The width of the object, if applicable. This refers to the attribute in `object`.
    /// Since it is duplicate or irrelevant information in all cases, use the equivalent
    /// member in [`ObjectShape`] instead.
    #[deprecated(since = "0.10.0", note = "Use [`ObjectShape`] members instead")]
    pub width: f32,
    /// The height of the object, if applicable. This refers to the attribute in `object`.
    /// Since it is duplicate or irrelevant information in all cases, use the equivalent
    /// member in [`ObjectShape`] instead.
    #[deprecated(since = "0.10.0", note = "Use [`ObjectShape`] members instead")]
    pub height: f32,
    /// The X coordinate of this object in pixels.
    pub x: f32,
    /// The Y coordinate of this object in pixels.
    pub y: f32,
    /// The clockwise rotation of this object around (x,y) in degrees.
    pub rotation: f32,
    /// Whether the object is shown or hidden.
    pub visible: bool,
    /// The object's shape.
    pub shape: ObjectShape,
    /// The object's custom properties as set by the user.
    pub properties: Properties,
}

impl ObjectData {
    /// ID of the object, which is unique per map since Tiled 0.11.
    ///
    /// On older versions this value is defaulted to 0.
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the data of the tile that this object is referencing, if it exists.
    #[inline]
    pub fn tile_data(&self) -> Option<ObjectTileData> {
        self.tile.clone()
    }
}

impl ObjectData {
    /// If it is known that the object has no tile images in it (i.e. collision data)
    /// then we can pass in [`None`] as the tilesets
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tilesets: Option<&[MapTilesetGid]>,
        for_tileset: Option<Arc<Tileset>>,
        // Base path is a directory to which all other files are relative to
        base_path: &Path,
        reader: &mut impl ResourceReader,
        cache: &mut impl ResourceCache,
    ) -> Result<ObjectData> {
        let (id, mut tile, x, y, mut n, mut t, mut w, mut h, mut v, mut r, template) = get_attrs!(
            attrs,
            optionals: [
                ("id", id, |v:String| v.parse().ok()),
                ("gid", tile, |v:String| v.parse().ok()
                                            .and_then(|bits| ObjectTileData::from_bits(bits, tilesets?, for_tileset.as_ref().cloned()))),
                ("x", x, |v:String| v.parse().ok()),
                ("y", y, |v:String| v.parse().ok()),
                ("name", name, |v:String| v.parse().ok()),
                ("type", obj_type, |v:String| v.parse().ok()),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("rotation", rotation, |v:String| v.parse().ok()),
                ("template", template, |v:String| v.parse().ok()),
            ]
        );

        // If the template attribute is there, we need to go fetch the template file
        let template = template
            .map(|template_path: String| {
                let template_path = base_path.join(Path::new(&template_path));

                // Check the cache to see if this template exists
                let template = if let Some(templ) = cache.get_template(&template_path) {
                    templ
                } else {
                    let template = Template::parse_template(&template_path, reader, cache)?;
                    // Insert it into the cache
                    cache.insert_template(&template_path, template.clone());
                    template
                };

                // The template sets the default values for the object
                let obj = &template.object;
                v.get_or_insert(obj.visible);
                #[allow(deprecated)]
                w.get_or_insert(obj.width);
                #[allow(deprecated)]
                h.get_or_insert(obj.height);
                r.get_or_insert(obj.rotation);
                n.get_or_insert_with(|| obj.name.clone());
                t.get_or_insert_with(|| obj.obj_type.clone());
                if let Some(templ_tile) = &obj.tile {
                    tile.get_or_insert_with(|| templ_tile.clone());
                }
                Ok(template)
            })
            .transpose()?;

        let x = x.unwrap_or(0f32);
        let y = y.unwrap_or(0f32);
        let visible = v.unwrap_or(true);
        let width = w.unwrap_or(0f32);
        let height = h.unwrap_or(0f32);
        let rotation = r.unwrap_or(0f32);
        let id = id.unwrap_or(0u32);
        let name = n.unwrap_or_default();
        let obj_type = t.unwrap_or_default();
        let mut shape = None;
        let mut properties = HashMap::new();

        parse_tag!(parser, "object", {
            "ellipse" => |_| {
                shape = Some(ObjectShape::Ellipse {
                    width,
                    height,
                });
                Ok(())
            },
            "polyline" => |attrs| {
                shape = Some(ObjectData::new_polyline(attrs)?);
                Ok(())
            },
            "polygon" => |attrs| {
                shape = Some(ObjectData::new_polygon(attrs)?);
                Ok(())
            },
            "point" => |_| {
                shape = Some(ObjectShape::Point(x, y));
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        // Possibly copy properties from the template into the object
        // Any that already exist in the object's map don't get copied over
        if let Some(templ) = template {
            shape.get_or_insert(templ.object.shape.clone());

            for (k, v) in &templ.object.properties {
                if !properties.contains_key(k) {
                    properties.insert(k.clone(), v.clone());
                }
            }
        }

        let shape = shape.unwrap_or(ObjectShape::Rect { width, height });

        #[allow(deprecated)]
        Ok(ObjectData {
            id,
            tile,
            name,
            obj_type,
            width,
            height,
            x,
            y,
            rotation,
            visible,
            shape,
            properties,
        })
    }
}

impl ObjectData {
    fn new_polyline(attrs: Vec<OwnedAttribute>) -> Result<ObjectShape> {
        let s = get_attrs!(
            attrs,
            required: [
                ("points", points, Some),
            ],
            Error::MalformedAttributes("A polyline must have points".to_string())
        );
        let points = ObjectData::parse_points(s)?;
        Ok(ObjectShape::Polyline { points })
    }

    fn new_polygon(attrs: Vec<OwnedAttribute>) -> Result<ObjectShape> {
        let s = get_attrs!(
            attrs,
            required: [
                ("points", points, Some),
            ],
            Error::MalformedAttributes("A polygon must have points".to_string())
        );
        let points = ObjectData::parse_points(s)?;
        Ok(ObjectShape::Polygon { points })
    }

    fn parse_points(s: String) -> Result<Vec<(f32, f32)>> {
        let pairs = s.split(' ');
        pairs
            .map(|point| point.split(','))
            .map(|components| {
                let v: Vec<&str> = components.collect();
                if v.len() != 2 {
                    return Err(Error::MalformedAttributes(
                        "one of a polyline's points does not have an x and y coordinate"
                            .to_string(),
                    ));
                }
                let (x, y) = (v[0].parse().ok(), v[1].parse().ok());
                match (x, y) {
                    (Some(x), Some(y)) => Ok((x, y)),
                    _ => Err(Error::MalformedAttributes(
                        "one of polyline's points does not have i32eger coordinates".to_string(),
                    )),
                }
            })
            .collect()
    }
}

map_wrapper!(
    #[doc = "Wrapper over an [`ObjectData`] that contains both a reference to the data as well as
    to the map it is contained in."]
    Object => ObjectData
);

impl<'map> Object<'map> {
    /// Returns the tile that the object is using as image, if any.
    pub fn get_tile(&self) -> Option<ObjectTile<'map>> {
        self.data
            .tile
            .as_ref()
            .map(|tile| ObjectTile::new(self.map, tile))
    }
}
