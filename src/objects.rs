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
    /// ID of the object, which is unique per map since Tiled 0.11.
    ///
    /// On older versions this value is defaulted to 0.
    pub id: u32,
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
    /// The object's custom properties set by the user.
    pub properties: Properties,
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
            attrs,
            optionals: [
                ("id", id, |v:String| v.parse().ok()),
                ("gid", tile, |v:String| v.parse().ok()
                                            .and_then(|bits| LayerTileData::from_bits(bits, tilesets?))),
                ("name", name, |v:String| v.parse().ok()),
                ("type", obj_type, |v:String| v.parse().ok()),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("rotation", rotation, |v:String| v.parse().ok()),
            ],
            required: [
                ("x", x, |v:String| v.parse().ok()),
                ("y", y, |v:String| v.parse().ok()),
            ],
            Error::MalformedAttributes("objects must have an x and a y number".to_string())
        );
        let visible = v.unwrap_or(true);
        let width = w.unwrap_or(0f32);
        let height = h.unwrap_or(0f32);
        let rotation = r.unwrap_or(0f32);
        let id = id.unwrap_or(0u32);
        let name = n.unwrap_or_else(|| String::new());
        let obj_type = t.unwrap_or_else(|| String::new());
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
        let ((), s) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("points", points, |v| Some(v)),
            ],
            Error::MalformedAttributes("A polyline must have points".to_string())
        );
        let points = ObjectData::parse_points(s)?;
        Ok(ObjectShape::Polyline { points })
    }

    fn new_polygon(attrs: Vec<OwnedAttribute>) -> Result<ObjectShape> {
        let ((), s) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("points", points, |v| Some(v)),
            ],
            Error::MalformedAttributes("A polygon must have points".to_string())
        );
        let points = ObjectData::parse_points(s)?;
        Ok(ObjectShape::Polygon { points: points })
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
    /// ID of the object, which is unique per map since Tiled 0.11.
    ///
    /// On older versions this value is defaulted to 0.
    #[inline]
    pub fn id(&self) -> u32 {
        self.data.id
    }

    /// Returns the tile that the object is using as image, if any.
    pub fn get_tile(&self) -> Option<LayerTile<'map>> {
        self.data
            .tile
            .as_ref()
            .map(|tile| LayerTile::new(self.map, tile))
    }

    /// The name of the object, which is arbitrary and set by the user.
    #[inline]
    pub fn name(&self) -> &str {
        self.data.name.as_ref()
    }

    /// The type of the object, which is arbitrary and set by the user.
    #[inline]
    pub fn obj_type(&self) -> &str {
        self.data.obj_type.as_ref()
    }

    /// The width of the object, if applicable. This refers to the attribute in `object`.
    /// Since it is duplicate or irrelevant information in all cases, use the equivalent
    /// member in [`ObjectShape`] instead.
    #[deprecated(since = "0.10.0", note = "Use [`ObjectShape`] members instead")]
    #[inline]
    pub fn width(&self) -> f32 {
        #[allow(deprecated)]
        self.data.width
    }

    /// The height of the object, if applicable. This refers to the attribute in `object`.
    /// Since it is duplicate or irrelevant information in all cases, use the equivalent
    /// member in [`ObjectShape`] instead.
    #[deprecated(since = "0.10.0", note = "Use [`ObjectShape`] members instead")]
    #[inline]
    pub fn height(&self) -> f32 {
        #[allow(deprecated)]
        self.data.height
    }

    /// The X coordinate of this object in pixels.
    #[inline]
    pub fn x(&self) -> f32 {
        self.data.x
    }

    /// The Y coordinate of this object in pixels.
    #[inline]
    pub fn y(&self) -> f32 {
        self.data.y
    }

    /// The clockwise rotation of this object around (x,y) in degrees.
    #[inline]
    pub fn rotation(&self) -> f32 {
        self.data.rotation
    }

    /// Whether the object is shown or hidden.
    #[inline]
    pub fn visible(&self) -> bool {
        self.data.visible
    }

    /// The object's shape.
    #[inline]
    pub fn shape(&self) -> &ObjectShape {
        &self.data.shape
    }

    /// The object's custom properties set by the user.
    #[inline]
    pub fn properties(&self) -> &Properties {
        &self.data.properties
    }
}
