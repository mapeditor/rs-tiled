use std::collections::HashMap;

use ggez::{
    graphics::{self, spritebatch::SpriteBatch, DrawParam},
    Context, GameResult,
};
use tiled::TileLayer;

pub struct MapHandler {
    map: tiled::Map,
    tileset_image_cache: HashMap<String, graphics::Image>,
    batch_cache: Option<HashMap<u32, Vec<SpriteBatch>>>,
    pub example_animate: bool,
}

impl MapHandler {
    pub fn new(map: tiled::Map, ctx: &mut Context) -> GameResult<Self> {
        // Load the map's tileset images
        let mut tileset_image_cache = HashMap::new();
        for ts in map.tilesets().iter() {
            if let Some(image) = &ts.image {
                let mut img = graphics::Image::new(ctx, &image.source)?;
                // Set filter to nearest to get crispy pixel art goodness
                img.set_filter(graphics::FilterMode::Nearest);

                tileset_image_cache.insert(ts.name.clone(), img);
            }
        }

        Ok(Self {
            tileset_image_cache,
            map,
            batch_cache: None,
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

    pub fn bounds(&self) -> graphics::Rect {
        graphics::Rect::new(
            0.0,
            0.0,
            (self.width() * self.tile_width()) as f32,
            (self.height() * self.tile_height()) as f32,
        )
    }

    pub fn background_color(&self) -> Option<graphics::Color> {
        self.map
            .background_color
            .map(|c| ggez::graphics::Color::from_rgba(c.red, c.green, c.blue, c.alpha))
    }

    /// Clear the tile layer `SpriteBatch` cache.
    /// Required if the tile instances change in any way (e.g. The tile positions change)
    pub fn invalidate_batch_cache(&mut self) {
        self.batch_cache = None;
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        draw_param: DrawParam,
        parallax_pan: (f32, f32),
    ) -> GameResult {
        // Update batch cache if needed

        if self.example_animate {
            // If it's animating, the individual tile positions are changing, so we can't use this cache
            self.invalidate_batch_cache();
        }

        // (Can't use `get_or_insert_with` due to needing to double borrow self)
        if self.batch_cache.is_none() {
            self.batch_cache = Some(self.generate_map_render(ctx, parallax_pan));
        }

        let layer_batches: &HashMap<u32, Vec<SpriteBatch>> = self.batch_cache.as_ref().unwrap();

        // Draw layers

        for l in self.map.layers() {
            match &l.layer_type() {
                tiled::LayerType::Objects(ol) => {
                    for o in ol.objects() {
                        Self::draw_object(&o, ctx, draw_param.clone())?;
                    }
                }
                tiled::LayerType::Tiles(_tl) => {
                    let batches = layer_batches.get(&l.id()).unwrap();

                    // Each tileset in the layer gets a different batch
                    for batch in batches {
                        graphics::draw(ctx, batch, draw_param)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Generates a set of `SpriteBatch`es for each tile layer in the map.
    fn generate_map_render(
        &self,
        ctx: &Context,
        parallax_pan: (f32, f32),
    ) -> HashMap<u32, Vec<SpriteBatch>> {
        let mut layer_batches: HashMap<u32, Vec<SpriteBatch>> = HashMap::new();

        let tile_layers = self.map.layers().filter_map(|l| match l.layer_type() {
            tiled::LayerType::Tiles(tl) => Some((l, tl)),
            _ => None,
        });

        for (i, (layer, tl)) in tile_layers.enumerate() {
            match &tl {
                TileLayer::Finite(d) => {
                    // Create a sprite batch for each tileset
                    // This needs to be done per layer otherwise the depth will be wrong when using tilesets on multiple layers
                    let mut ts_sizes_and_batches = HashMap::new();
                    for ts in self.map.tilesets().iter() {
                        if let Some(img) = self.tileset_image_cache.get(&ts.name) {
                            // img.clone() here is cheap, as it is just an Arc'ed handle (see docs for `ggez::graphics::Image`)
                            let batch = SpriteBatch::new(img.clone());
                            ts_sizes_and_batches
                                .insert(ts.name.clone(), (batch, (img.width(), img.height())));
                        }
                    }

                    let width = d.width();
                    let height = d.height();

                    let secs_since_start = ggez::timer::time_since_start(ctx).as_secs_f32();

                    // Iterate through every tile in the layer
                    for x in 0..width as i32 {
                        for y in 0..height as i32 {
                            if let Some(tile) = d.get_tile(x, y) {
                                // Get tile's rectangle in the tileset texture
                                let ts = tile.get_tileset();
                                if let Some((batch, ts_size)) =
                                    ts_sizes_and_batches.get_mut(&ts.name)
                                {
                                    let mut dx = x as f32 * self.map.tile_width as f32
                                        + parallax_pan.0 * (layer.parallax_x - 1.0);
                                    let mut dy = y as f32 * self.map.tile_height as f32
                                        + parallax_pan.1 * (layer.parallax_y - 1.0);

                                    if self.example_animate {
                                        dx += (secs_since_start - x as f32 * 0.3 + i as f32 * 0.25)
                                            .sin()
                                            * 20.0;
                                        dy += (secs_since_start * 1.25
                                            + y as f32 * 0.3
                                            + i as f32 * 0.25)
                                            .cos()
                                            * 20.0;
                                    }

                                    batch.add(
                                        DrawParam::default()
                                            .src(get_tile_rect(ts, tile.id(), ts_size.0, ts_size.1))
                                            .dest([dx, dy])
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

                    layer_batches.insert(
                        layer.id(),
                        ts_sizes_and_batches.into_values().map(|sb| sb.0).collect(),
                    );
                }
                TileLayer::Infinite(_) => {
                    // Repeat the same process, but make use of chunks as well...
                    // A bit more complicated, so for simplicity's sake, not implemented here
                    unimplemented!()
                }
            }
        }

        layer_batches
    }

    fn draw_object(
        object: &tiled::ObjectData,
        ctx: &mut Context,
        draw_param: DrawParam,
    ) -> GameResult {
        match &object.shape {
            tiled::ObjectShape::Rect { width, height } => {
                let bounds = graphics::Rect::new(object.x, object.y, *width, *height);
                let shape = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(2.0),
                    bounds,
                    graphics::Color::CYAN,
                )?;
                graphics::draw(ctx, &shape, draw_param)?;
            }
            tiled::ObjectShape::Ellipse { width, height } => {
                let shape = graphics::Mesh::new_ellipse(
                    ctx,
                    graphics::DrawMode::stroke(2.0),
                    [object.x + width / 2.0, object.y + height / 2.0],
                    *width / 2.0,
                    *height / 2.0,
                    0.5,
                    graphics::Color::CYAN,
                )?;
                graphics::draw(ctx, &shape, draw_param)?;
            }
            tiled::ObjectShape::Polyline { points } => {
                let points: Vec<_> = points
                    .iter()
                    .map(|p| [p.0 + object.x, p.1 + object.y])
                    .collect();
                let shape = graphics::Mesh::new_polyline(
                    ctx,
                    graphics::DrawMode::stroke(2.0),
                    &points,
                    graphics::Color::CYAN,
                )?;
                graphics::draw(ctx, &shape, draw_param)?;
            }
            tiled::ObjectShape::Polygon { points } => {
                let points: Vec<_> = points
                    .iter()
                    .map(|p| [p.0 + object.x, p.1 + object.y])
                    .collect();
                let shape = graphics::Mesh::new_polyline(
                    ctx,
                    graphics::DrawMode::stroke(2.0),
                    &points,
                    graphics::Color::CYAN,
                )?;
                graphics::draw(ctx, &shape, draw_param)?;
            }
            tiled::ObjectShape::Point(_, _) | tiled::ObjectShape::Text { .. } => {
                // Left as an exercise for the reader
            }
        }

        if !object.name.is_empty() {
            let text = graphics::Text::new(object.name.clone());
            graphics::queue_text(
                ctx,
                &text,
                [object.x, object.y],
                Some(graphics::Color::YELLOW),
            );
            graphics::draw_queued_text(ctx, draw_param, None, graphics::FilterMode::Nearest)?;
        }

        Ok(())
    }
}

fn get_tile_rect(
    tileset: &tiled::Tileset,
    id: u32,
    ts_img_width: u16,
    ts_img_height: u16,
) -> graphics::Rect {
    let ts_x = id % tileset.columns;
    let ts_y = id / tileset.columns;

    let x = (tileset.margin + (tileset.tile_width + tileset.spacing) * ts_x) as f32;
    let y = (tileset.margin + (tileset.tile_height + tileset.spacing) * ts_y) as f32;

    let ts_img_width = ts_img_width as f32;
    let ts_img_height = ts_img_height as f32;

    graphics::Rect {
        x: x / ts_img_width,
        y: y / ts_img_height,
        w: tileset.tile_width as f32 / ts_img_width,
        h: tileset.tile_height as f32 / ts_img_height,
    }
}
