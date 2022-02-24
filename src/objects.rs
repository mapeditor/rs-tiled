use std::{collections::HashMap, path::Path};

use xml::attribute::OwnedAttribute;

use crate::{
    error::TiledError,
    properties::{parse_properties, Properties},
    template::Template,
    util::{get_attrs, map_wrapper, parse_tag, XmlEventResult},
    LayerTile, LayerTileData, MapTilesetGid, ResourceCache,
};

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectShape {
    Rect { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Polyline { points: Vec<(f32, f32)> },
    Polygon { points: Vec<(f32, f32)> },
    Point(f32, f32),
}

/// Raw data belonging to an object. Used internally and for tile collisions.
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
        templates: &mut Vec<Template>,
        for_template: Option<usize>,
        base_path: &Path,
        cache: &mut impl ResourceCache,
    ) -> Result<ObjectData, TiledError> {
        let (
            (mut id, mut tile, mut x, mut y, mut n, mut t, mut w, mut h, mut v, mut r, template),
            (),
        ) = get_attrs!(
            attrs,
            optionals: [
                ("id", id, |v:String| v.parse().ok()),
                ("gid", tile, |v:String| v.parse().ok()
                                            .and_then(|bits| LayerTileData::from_bits(bits, tilesets?, for_template))),
                ("x", x, |v:String| v.parse().ok()),
                ("y", y, |v:String| v.parse().ok()),
                ("name", name, |v:String| v.parse().ok()),
                ("type", obj_type, |v:String| v.parse().ok()),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
                ("visible", visible, |v:String| v.parse().ok().map(|x:i32| x == 1)),
                ("rotation", rotation, |v:String| v.parse().ok()),
                ("template", template, |v:String| v.parse().ok()),
            ],
            required: [],
            TiledError::MalformedAttributes("objects must have an x and a y number".to_string())
        );

        // If the template attribute is there, we need to go fetch the template file
        let template_id = if let Some(template) = template {
            let s: String = template;
            let parent_dir = base_path.parent().unwrap();
            let template_path = parent_dir.join(Path::new(&s));

            let template_id =
                Template::parse_and_append_template(&template_path, templates, cache)?;

            // The template sets the default values for the object
            if let Some(obj) = &templates[template_id].object {
                x.get_or_insert(obj.x);
                y.get_or_insert(obj.y);
                v.get_or_insert(obj.visible);
                w.get_or_insert(obj.width);
                h.get_or_insert(obj.height);
                r.get_or_insert(obj.rotation);
                id.get_or_insert(obj.id);
                n.get_or_insert(obj.name.clone());
                t.get_or_insert(obj.obj_type.clone());
                if let Some(templ_tile) = obj.tile {
                    tile.get_or_insert(templ_tile);
                }
            };
            Some(template_id)
        } else {
            None
        };

        let x = x.unwrap_or(0f32);
        let y = y.unwrap_or(0f32);
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

        // Possibly copy properties from the template into the object
        // Any that already exist in the object's map don't get copied over
        if let Some(id) = template_id.as_ref() {
            if let Some(obj) = &templates[*id].object {
                for (k, v) in &obj.properties {
                    if !properties.contains_key(k) {
                        properties.insert(k.clone(), v.clone());
                    }
                }
            }
        }

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
        Ok(ObjectShape::Polyline { points })
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

    fn parse_points(s: String) -> Result<Vec<(f32, f32)>, TiledError> {
        let pairs = s.split(' ');
        pairs
            .map(|point| point.split(','))
            .map(|components| {
                let v: Vec<&str> = components.collect();
                if v.len() != 2 {
                    return Err(TiledError::MalformedAttributes(
                        "one of a polyline's points does not have an x and y coordinate"
                            .to_string(),
                    ));
                }
                let (x, y) = (v[0].parse().ok(), v[1].parse().ok());
                match (x, y) {
                    (Some(x), Some(y)) => Ok((x, y)),
                    _ => Err(TiledError::MalformedAttributes(
                        "one of polyline's points does not have i32eger coordinates".to_string(),
                    )),
                }
            })
            .collect()
    }
}

map_wrapper!(Object => ObjectData);

impl<'map> Object<'map> {
    /// Get the object's id.
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

    /// Get a reference to the object's name.
    pub fn name(&self) -> &str {
        self.data.name.as_ref()
    }

    /// Get a reference to the object's type.
    pub fn obj_type(&self) -> &str {
        self.data.obj_type.as_ref()
    }

    /// Get the object's width.
    pub fn width(&self) -> f32 {
        self.data.width
    }

    /// Get the object's height.
    pub fn height(&self) -> f32 {
        self.data.height
    }

    /// Get the object's x.
    pub fn x(&self) -> f32 {
        self.data.x
    }

    /// Get object's y.
    pub fn y(&self) -> f32 {
        self.data.y
    }

    /// Get a reference to the object's rotation.
    pub fn rotation(&self) -> f32 {
        self.data.rotation
    }

    /// Whether the object should be visible or not.
    pub fn visible(&self) -> bool {
        self.data.visible
    }

    /// Get a reference to the object's shape.
    pub fn shape(&self) -> &ObjectShape {
        &self.data.shape
    }

    /// Get a reference to the object's properties.
    pub fn properties(&self) -> &Properties {
        &self.data.properties
    }
}
