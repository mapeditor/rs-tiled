use std::{collections::HashMap, path::PathBuf};

use ggez::{graphics::{self, spritebatch::SpriteBatch, DrawParam}, Context, GameResult};
use tiled::{TileLayer, Tileset};

pub struct MapHandler {
    map: tiled::Map,
    tileset_image_cache: HashMap<String, graphics::Image>,
    pub example_animate: bool,
}

impl MapHandler {
    pub fn new(map: tiled::Map, ctx: &mut Context) -> GameResult<Self> {
        // load images for the map's tilesets
        let mut tileset_image_cache = HashMap::new();
        for ts in map.tilesets().iter() {
            if let Some(image) = &ts.image {
                // image path comes in as "assets/tilesheet.png"
                // but ggez needs it like "/assets/tilesheet.png" or it will complain
                let mut pb = PathBuf::new();
                pb.push("/");
                pb.push(image.source.clone());
                
                // load the image
                let mut img = graphics::Image::new(ctx, pb)?;
                img.set_filter(graphics::FilterMode::Nearest);

                tileset_image_cache.insert(
                    ts.name.clone(),
                    img,
                );
            }
        }

        Ok(Self {
            tileset_image_cache,
            map,
            example_animate: false,
        })
    }

    pub fn width(&self) -> u32 {
        self.map.width
    }

    pub fn height(&self) -> u32 {
        self.map.height
    }

    pub fn tile_width(&self) -> u32 {
        self.map.tile_width
    }

    pub fn tile_height(&self) -> u32 {
        self.map.tile_height
    }

    pub fn get_bounds(&self) -> graphics::Rect {
        graphics::Rect::new(0.0, 0.0, (self.height() * self.tile_height()) as f32, (self.height() * self.tile_height()) as f32)
    }

    pub fn background_color(&self) -> Option<graphics::Color> {
        self.map.background_color
            .map(|c| ggez::graphics::Color::from_rgba(c.red, c.green, c.blue, c.alpha))
    }

    pub fn draw(&mut self, ctx: &mut Context, draw_param: DrawParam, parallax_pan: (f32, f32)) -> GameResult {

        let layer_batches: Vec<Vec<SpriteBatch>> = self.generate_map_render(ctx, parallax_pan);
        
        // draw tile layers

        // for each layer
        for layer in &layer_batches {
            // for each tileset in the layer
            for batch in layer {
                graphics::draw(ctx, batch, draw_param)?;
            }
        }

        // draw objects

        for l in self.map.layers() {
            match &l.layer_type() {
                tiled::LayerType::ObjectLayer(ol) => {
                    for o in &ol.data().objects {
                        Self::draw_object(o, ctx, draw_param.clone())?;
                    }
                }
                _ => {},
            }
        }

        Ok(())
    }

    fn generate_map_render(&mut self, ctx: &Context, parallax_pan: (f32, f32)) -> Vec<Vec<SpriteBatch>> {
        let mut layer_batches: Vec<Vec<SpriteBatch>> = Vec::new();

        let tile_layers = self.map.layers().filter_map(|l| {
            match l.layer_type() {
                tiled::LayerType::TileLayer(tl) => Some((l.data(), tl)),
                _ => None,
            }
        });

        for (i, (layer, tl)) in tile_layers.enumerate() {
            match &tl {
                TileLayer::Finite(d) => {
                    // create a sprite batch for each tileset
                    // this needs to be done per layer otherwise the depth will be wrong when using tilesets on multiple layers
                    let mut ts_sizes_and_batches = HashMap::new();
                    for ts in self.map.tilesets().iter() {
                        if let Some(img) = self.tileset_image_cache.get(&ts.name) {
                            // img.clone() here is cheap (see docs for `ggez::graphics::Image`)
                            let batch = SpriteBatch::new(img.clone());
                            ts_sizes_and_batches.insert(ts.name.clone(), (batch, (img.width(), img.height())));
                        }
                    }
                    
                    let width = d.data().width();
                    let height = d.data().height();

                    // iterate through every tile in the layer
                    for x in 0..width as i32 {
                        for y in 0..height as i32 {
                            if let Some(tile) = d.get_tile(x, y) {
                                // get tile's rectangle in the tileset texture
                                let rect = get_tile_rect(tile.tileset, tile.id);
                                let ts = tile.tileset;
                                if let Some((batch, ts_size)) = ts_sizes_and_batches.get_mut(&ts.name) {
                                    let mut dx = x as f32 * self.map.tile_width as f32 + parallax_pan.0 * (layer.parallax_x - 1.0);
                                    let mut dy = y as f32 * self.map.tile_height as f32 + parallax_pan.1 * (layer.parallax_y - 1.0);

                                    if self.example_animate {
                                        dx += (ggez::timer::time_since_start(ctx).as_secs_f32() - x as f32 * 0.3 + i as f32 * 0.25).sin() * 20.0;
                                        dy += (ggez::timer::time_since_start(ctx).as_secs_f32() * 1.25 + y as f32 * 0.3 + i as f32 * 0.25).cos() * 20.0;
                                    }

                                    batch.add(
                                        DrawParam::default()
                                            .src(ggez::graphics::Rect::new(
                                                rect.0 as f32 / (*ts_size).0 as f32,
                                                rect.1 as f32 / (*ts_size).1 as f32,
                                                rect.2 as f32 / (*ts_size).0 as f32,
                                                rect.3 as f32 / (*ts_size).1 as f32,
                                            ))
                                            .dest([
                                                dx,
                                                dy,
                                            ])
                                            .color(ggez::graphics::Color::from_rgba(
                                                0xFF,
                                                0xFF,
                                                0xFF,
                                                (layer.opacity * 255.0) as u8,
                                            )),
                                    );
                                }
                            }
                        }
                    }

                    layer_batches.push(ts_sizes_and_batches.into_values().map(|sb| sb.0).collect());
                }
                TileLayer::Infinite(_) => {
                    unimplemented!()
                }
            }
        }

        layer_batches
    }
    
    fn draw_object(object: &tiled::ObjectData, ctx: &mut Context, draw_param: DrawParam) -> GameResult {
        match &object.shape {
            tiled::ObjectShape::Rect { width, height } => {
                let bounds = graphics::Rect::new(object.x, object.y, *width, *height);
                let shape =
                    graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::stroke(2.0), bounds, graphics::Color::CYAN)?;
                graphics::draw(ctx, &shape, draw_param)?;
            },
            tiled::ObjectShape::Ellipse { width, height } => {
                let shape = graphics::Mesh::new_ellipse(
                    ctx, 
                    graphics::DrawMode::stroke(2.0), 
                    [object.x + width / 2.0, object.y + height / 2.0], 
                    *width / 2.0, 
                    *height / 2.0, 
                    0.5, 
                    graphics::Color::CYAN
                )?;
                graphics::draw(ctx, &shape, draw_param)?;
            },
            tiled::ObjectShape::Polyline { points } => {
                let points: Vec<_> = points.iter().map(|p| [p.0 + object.x, p.1 + object.y]).collect();
                let shape = graphics::Mesh::new_polyline(
                    ctx, 
                    graphics::DrawMode::stroke(2.0), 
                    &points,
                    graphics::Color::CYAN
                )?;
                graphics::draw(ctx, &shape, draw_param)?;
            },
            tiled::ObjectShape::Polygon { points } => {
                let points: Vec<_> = points.iter().map(|p| [p.0 + object.x, p.1 + object.y]).collect();
                let shape = graphics::Mesh::new_polyline(
                    ctx, 
                    graphics::DrawMode::stroke(2.0),
                    &points,
                    graphics::Color::CYAN
                )?;
                graphics::draw(ctx, &shape, draw_param)?;
            },
            tiled::ObjectShape::Point(_, _) => {
                // exercise for the reader
            },
        }

        if !object.name.is_empty() {
            let text = graphics::Text::new(object.name.clone());
            graphics::queue_text(ctx, &text, [object.x, object.y], Some(graphics::Color::YELLOW));
            graphics::draw_queued_text(ctx, draw_param, None, graphics::FilterMode::Nearest)?;
        }

        Ok(())
    }
}

fn get_tile_rect(tileset: &Tileset, id: u32) -> (u32, u32, u32, u32) {
    let ts_x = id % tileset.columns;
    let ts_y = id / tileset.columns;

    let x = tileset.margin + (tileset.tile_width + tileset.spacing) * ts_x;
    let y = tileset.margin + (tileset.tile_height + tileset.spacing) * ts_y;

    (x, y, tileset.tile_width, tileset.tile_height)
}