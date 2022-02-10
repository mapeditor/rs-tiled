use std::collections::HashMap;

use xml::attribute::OwnedAttribute;

use crate::{
    error::TiledError,
    properties::{parse_properties, Properties},
    util::{get_attrs, parse_tag, XmlEventResult},
    LayerTile, LayerTileData, MapTilesetGid, MapWrapper,
};

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectShape {
    Rect { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Polyline { points: Vec<(f32, f32)> },
    Polygon { points: Vec<(f32, f32)> },
    Point(f32, f32),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectData {
    pub id: u32,
    tile: Option<LayerTileData>,
    pub name: String,
    pub obj_type: String,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub visible: bool,
    pub shape: ObjectShape,
    pub properties: Properties,
}

impl ObjectData {
    /// If it is known that the object has no tile images in it (i.e. collision data)
    /// then we can pass in [`None`] as the tilesets
    pub(crate) fn new(
        parser: &mut impl Iterator<Item = XmlEventResult>,
        attrs: Vec<OwnedAttribute>,
        tilesets: Option<&[MapTilesetGid]>,
    ) -> Result<ObjectData, TiledError> {
        let ((id, bits, n, t, w, h, v, r), (x, y)) = get_attrs!(
            attrs,
            optionals: [
                ("id", id, |v:String| v.parse().ok()),
                ("gid", gid, |v:String| v.parse().ok()),
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
            TiledError::MalformedAttributes("objects must have an x and a y number".to_string())
        );
        let v = v.unwrap_or(true);
        let w = w.unwrap_or(0f32);
        let h = h.unwrap_or(0f32);
        let r = r.unwrap_or(0f32);
        let id = id.unwrap_or(0u32);
        let tile = bits.and_then(|bits| LayerTileData::from_bits(bits, tilesets?));
        let n = n.unwrap_or(String::new());
        let t = t.unwrap_or(String::new());
        let mut shape = None;
        let mut properties = HashMap::new();

        parse_tag!(parser, "object", {
            "ellipse" => |_| {
                shape = Some(ObjectShape::Ellipse {
                    width: w,
                    height: h,
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
                shape = Some(ObjectData::new_point(x, y)?);
                Ok(())
            },
            "properties" => |_| {
                properties = parse_properties(parser)?;
                Ok(())
            },
        });

        let shape = shape.unwrap_or(ObjectShape::Rect {
            width: w,
            height: h,
        });

        Ok(ObjectData {
            id,
            tile,
            name: n.clone(),
            obj_type: t.clone(),
            width: w,
            height: h,
            x,
            y,
            rotation: r,
            visible: v,
            shape,
            properties,
        })
    }
}

impl ObjectData {
    fn new_polyline(attrs: Vec<OwnedAttribute>) -> Result<ObjectShape, TiledError> {
        let ((), s) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("points", points, |v| Some(v)),
            ],
            TiledError::MalformedAttributes("A polyline must have points".to_string())
        );
        let points = ObjectData::parse_points(s)?;
        Ok(ObjectShape::Polyline { points: points })
    }

    fn new_polygon(attrs: Vec<OwnedAttribute>) -> Result<ObjectShape, TiledError> {
        let ((), s) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("points", points, |v| Some(v)),
            ],
            TiledError::MalformedAttributes("A polygon must have points".to_string())
        );
        let points = ObjectData::parse_points(s)?;
        Ok(ObjectShape::Polygon { points: points })
    }

    fn new_point(x: f32, y: f32) -> Result<ObjectShape, TiledError> {
        Ok(ObjectShape::Point(x, y))
    }

    fn parse_points(s: String) -> Result<Vec<(f32, f32)>, TiledError> {
        let pairs = s.split(' ');
        let mut points = Vec::new();
        for v in pairs.map(|p| p.split(',')) {
            let v: Vec<&str> = v.collect();
            if v.len() != 2 {
                return Err(TiledError::MalformedAttributes(
                    "one of a polyline's points does not have an x and y coordinate".to_string(),
                ));
            }
            let (x, y) = (v[0].parse().ok(), v[1].parse().ok());
            if x.is_none() || y.is_none() {
                return Err(TiledError::MalformedAttributes(
                    "one of polyline's points does not have i32eger coordinates".to_string(),
                ));
            }
            points.push((x.unwrap(), y.unwrap()));
        }
        Ok(points)
    }
}

pub type Object<'map> = MapWrapper<'map, ObjectData>;

impl<'map> Object<'map> {
    /// Returns the tile that the object is using as image, if any.
    pub fn get_tile<'res: 'map>(&self) -> Option<LayerTile<'map>> {
        self.data()
            .tile
            .map(|tile| LayerTile::from_data(&tile, self.map()))
    }
}
