use std::{collections::HashMap, path::PathBuf};

use ggez::{event::{self, MouseButton}, Context, GameResult, graphics::{self, spritebatch::SpriteBatch, DrawParam}, input};
use tiled::{Map, LayerData};

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("tiled + ggez", "tiled")
        .window_setup(ggez::conf::WindowSetup::default()
            .title("tiled + ggez example")
            .vsync(false)
        )
        .window_mode(ggez::conf::WindowMode::default()
            .dimensions(1000.0, 800.0)
        )
        .add_resource_path(""); // add repo root to ggez filesystem (our example map looks for `assets/tilesheet.png`)
    let (mut ctx, event_loop) = cb.build()?;
    let state = Game::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}

struct Game {
    map: Map,
    tileset_image_cache: HashMap<u32, graphics::Image>,
    pan: (f32, f32),
    scale: f32,
    animate: bool,
}

impl Game {
    fn new(ctx: &mut ggez::Context) -> GameResult<Self> {
        graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);

        // load the map
        let map = Map::parse_file("assets/tiled_base64_external.tmx").unwrap();

        // load images for the map's tilesets
        let mut tileset_image_cache = HashMap::new();
        for ts in &map.tilesets {
            if let Some(image) = &ts.image {
                // image path comes in as "assets/tilesheet.png"
                // but ggez needs it like "/assets/tilesheet.png" or it will complain
                let mut pb: PathBuf = PathBuf::new();
                pb.push("/");
                pb.push(image.source.clone());
                
                // load the image
                let mut img = graphics::Image::new(ctx, pb)?;
                img.set_filter(graphics::FilterMode::Nearest);

                tileset_image_cache.insert(
                    ts.first_gid.clone(),
                    img,
                );
            }
        }

        Ok(Self {
            map,
            tileset_image_cache,
            pan: (0.0, 0.0),
            scale: 1.0,
            animate: false,
        })
    }

    fn generate_map_render(&mut self, ctx: &Context) -> Vec<Vec<SpriteBatch>> {
        let mut layer_batches: Vec<Vec<SpriteBatch>> = Vec::new();

        let tile_layers = self.map.layers.iter().filter_map(|l| {
            match &l.layer_type {
                tiled::LayerType::TileLayer(tl) => Some((l, tl)),
                _ => None,
            }
        });

        for (i, (layer, tl)) in tile_layers.enumerate() {
            match &tl.tiles {
                LayerData::Finite(d) => {
                    // create a sprite batch for each tileset
                    // this needs to be done per layer otherwise the depth will be wrong when using tilesets on multiple layers
                    let mut ts_batches = HashMap::new();
                    let mut ts_sizes = HashMap::new();
                    for ts in &self.map.tilesets {
                        if let Some(img) = self.tileset_image_cache.get(&ts.first_gid) {
                            // img.clone() here is cheap (see docs for `ggez::graphics::Image`)
                            let batch = SpriteBatch::new(img.clone());
                            ts_batches.insert(ts.first_gid, batch);
                            ts_sizes.insert(ts.first_gid, (img.width(), img.height()));
                        }
                    }

                    for (pos, tile) in d.iter().enumerate() {
                        let (x, y) = (pos % self.map.width as usize, pos / self.map.width as usize);
                        let rect = get_tile_rect(&self.map, tile.gid);
                        if let Some(rect) = rect {
                            if let Some(ts) = self.map.tileset_by_gid(tile.gid) {
                                if let Some(ts_size) = ts_sizes.get(&ts.first_gid) {
                                    let mut dx = x as f32 * self.map.tile_width as f32 + self.pan.0 * (layer.parallax_x - 1.0);
                                    let mut dy = y as f32 * self.map.tile_height as f32 + self.pan.1 * (layer.parallax_y - 1.0);

                                    if self.animate {
                                        dx += (ggez::timer::time_since_start(ctx).as_secs_f32() - x as f32 * 0.3 + i as f32 * 0.25).sin() * 20.0;
                                        dy += (ggez::timer::time_since_start(ctx).as_secs_f32() * 1.25 + y as f32 * 0.3 + i as f32 * 0.25).cos() * 20.0;
                                    }

                                    let b = ts_batches.get_mut(&ts.first_gid).unwrap();
                                    b.add(
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

                    layer_batches.push(ts_batches.into_values().collect());
                }
                LayerData::Infinite(_) => {
                    unimplemented!()
                }
            }
        }

        layer_batches
    }

    fn draw_object(object: &tiled::Object, ctx: &mut Context, draw_param: DrawParam) -> GameResult {
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

impl event::EventHandler<ggez::GameError> for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let bg_color: ggez::graphics::Color = self.map.background_color
            .map(|c| ggez::graphics::Color::from_rgba(c.red, c.green, c.blue, 0xFF))
            .unwrap_or([0.1, 0.2, 0.3, 1.0].into());

        graphics::clear(ctx, bg_color);

        let window_size = graphics::size(ctx);

        let layer_batches: Vec<Vec<SpriteBatch>> = self.generate_map_render(ctx);

        let draw_param = DrawParam::default()
            .dest([
                self.pan.0 + window_size.0 / 2.0 - (self.map.width * self.map.tile_width) as f32 / 2.0, 
                self.pan.1 + window_size.1 / 2.0 - (self.map.height * self.map.tile_height) as f32 / 2.0
                ])
            .scale([self.scale, self.scale]);

        // draw tile layers

        // for each layer
        for layer in &layer_batches {
            // for each tileset in the layer
            for batch in layer {
                graphics::draw(ctx, batch, draw_param)?;
            }
        }

        for l in &self.map.layers {
            match &l.layer_type {
                tiled::LayerType::ObjectLayer(ol) => {
                    for o in &ol.objects {
                        Self::draw_object(o, ctx, draw_param.clone())?;
                    }
                }
                _ => {},
            }
        }

        // draw map bounds

        let rect = graphics::Rect::new(0.0, 0.0, (self.map.height * self.map.tile_height) as f32, (self.map.height * self.map.tile_height) as f32);
        let r1 =
            graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::stroke(2.0 / self.scale), rect, graphics::Color::from_rgb_u32(0x888888))?;
        graphics::draw(ctx, &r1, draw_param)?;

        // draw fps

        let fps = ggez::timer::fps(ctx);
        let text = graphics::Text::new(format!("{fps:.0} fps"));

        graphics::draw(
            ctx,
            &text,
            DrawParam::default()
                .dest([window_size.0 - text.width(ctx) - 40.0, 10.0])
                .scale([1.25, 1.25])
                .color(graphics::Color::WHITE),
        )?;

        // present

        graphics::present(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: event::MouseButton, _x: f32, _y: f32) {
        if button == MouseButton::Right {
            self.animate = !self.animate;
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, _x: f32, _y: f32, dx: f32, dy: f32) {
        if input::mouse::button_pressed(ctx, event::MouseButton::Middle) {
            self.pan.0 += dx;
            self.pan.1 += dy;
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        let old_scale = self.scale;
        self.scale *= 1.0 + y as f32 * 0.1;

        // zoom to mouse cursor
        let mouse_pos = input::mouse::position(ctx);
        let window_size = graphics::size(ctx);
        let pos_x = mouse_pos.x - window_size.0 / 2.0 + (self.map.width * self.map.tile_width) as f32 / 2.0;
        let pos_y = mouse_pos.y - window_size.1 / 2.0 + (self.map.height * self.map.tile_height) as f32 / 2.0;
        self.pan.0 = (self.pan.0 - pos_x) / old_scale * self.scale + pos_x;
        self.pan.1 = (self.pan.1 - pos_y) / old_scale * self.scale + pos_y;
    }
}

fn get_tile_rect(map: &Map, gid: u32) -> Option<(u32, u32, u32, u32)> {
    let ts = map.tileset_by_gid(gid)?;
    let id = gid - ts.first_gid;

    let ts_x = id % ts.columns;
    let ts_y = id / ts.columns;

    let x = ts.margin + (ts.tile_width + ts.spacing) * ts_x;
    let y = ts.margin + (ts.tile_height + ts.spacing) * ts_y;

    Some((x, y, ts.tile_width, ts.tile_height))
}