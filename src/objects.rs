use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    error::{Error, Result},
    properties::{parse_properties, Properties},
    util::{get_attrs, map_wrapper, parse_tag, XmlEventResult},
    LayerTile, LayerTileData, MapTilesetGid,
};

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
    tile: Option<LayerTileData>,
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
    pub fn tile_data(&self) -> Option<LayerTileData> {
        self.tile
    }
}

impl ObjectData {
    /// If it is known that the object has no tile images in it (i.e. collision data)
    /// then we can pass in [`None`] as the tilesets
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tilesets: Option<&[MapTilesetGid]>,
    ) -> Result<ObjectData> {
        let ((id, tile, n, t, w, h, v, r), (x, y)) = get_attrs!(
            for v in attrs {
                Some("id") => id ?= v.parse(),
                Some("gid") => tile ?= v.parse(),
                Some("name") => name ?= v.parse(),
                Some("type") => obj_type ?= v.parse(),
                Some("width") => width ?= v.parse(),
                Some("height") => height ?= v.parse(),
                Some("visible") => visible ?= v.parse().map(|x:i32| x == 1),
                Some("rotation") => rotation ?= v.parse(),

                "x" => x ?= v.parse::<f32>(),
                "y" => y ?= v.parse::<f32>(),
            }
            ((id, tile, name, obj_type, width, height, visible, rotation), (x, y))
        );
        let tile = tile.and_then(|bits| LayerTileData::from_bits(bits, tilesets?));
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
        let points = get_attrs!(
            for v in attrs {
                "points" => points ?= ObjectData::parse_points(v),
            }
            points
        );
        Ok(ObjectShape::Polyline { points })
    }

    fn new_polygon(attrs: Vec<OwnedAttribute>) -> Result<ObjectShape> {
        let points = get_attrs!(
            for v in attrs {
                "points" => points ?= ObjectData::parse_points(v),
            }
            points
        );
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
    pub fn get_tile(&self) -> Option<LayerTile<'map>> {
        self.data
            .tile
            .as_ref()
            .map(|tile| LayerTile::new(self.map, tile))
    }
}
