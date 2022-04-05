use std::{collections::HashMap, path::Path, sync::Arc};

use xml::attribute::OwnedAttribute;

use crate::{
    error::{Error, Result},
    properties::{parse_properties, Properties},
    template::Template,
    util::{get_attrs, map_wrapper, parse_tag, XmlEventResult},
    LayerTile, LayerTileData, MapTilesetGid, ResourceCache, Tileset,
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
        base_path: &Path,
        cache: &mut impl ResourceCache,
    ) -> Result<ObjectData> {
        let (mut id, mut tile, mut x, mut y, mut n, mut t, mut w, mut h, mut v, mut r, template) = get_attrs!(
            attrs,
            optionals: [
                ("id", id, |v:String| v.parse().ok()),
                ("gid", tile, |v:String| v.parse().ok()
                                            .and_then(|bits| LayerTileData::from_bits(bits, tilesets?, for_tileset.as_ref().cloned()))),
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
            .map(|template| {
                let s: String = template;
                let parent_dir = base_path.parent().ok_or(Error::PathIsNotFile)?;
                let template_path = parent_dir.join(Path::new(&s));

                // Check the cache to see if this template exists
                let template = if let Some(templ) = cache.get_template(&template_path) {
                    templ
                } else {
                    let template = Template::parse_template(&template_path, cache)?;
                    // Insert it into the cache
                    cache.insert_template(&template_path, template.clone());
                    template
                };

                // The template sets the default values for the object
                let obj = &template.object;
                x.get_or_insert(obj.x);
                y.get_or_insert(obj.y);
                v.get_or_insert(obj.visible);
                #[allow(deprecated)]
                w.get_or_insert(obj.width);
                #[allow(deprecated)]
                h.get_or_insert(obj.height);
                r.get_or_insert(obj.rotation);
                id.get_or_insert(obj.id);
                n.get_or_insert(obj.name.clone());
                t.get_or_insert(obj.obj_type.clone());
                if let Some(templ_tile) = obj.tile.clone() {
                    tile.get_or_insert(templ_tile);
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
        let name = n.unwrap_or_else(String::new);
        let obj_type = t.unwrap_or_else(String::new);
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

        // Possibly copy properties from the template into the object
        // Any that already exist in the object's map don't get copied over
        if let Some(templ) = template {
            for (k, v) in &templ.object.properties {
                if !properties.contains_key(k) {
                    properties.insert(k.clone(), v.clone());
                }
            }
        }

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
    pub fn get_tile(&self) -> Option<LayerTile<'map>> {
        self.data
            .tile
            .as_ref()
            .map(|tile| LayerTile::new(self.map, tile))
    }
}
