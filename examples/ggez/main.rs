mod map;

use ggez::{event::{self, MouseButton}, Context, GameResult, graphics::{self, DrawParam}, input};
use map::MapHandler;
use tiled::{Map, FilesystemResourceCache};

fn main() -> GameResult {
    // init ggez
    let cb = ggez::ContextBuilder::new("rs-tiled + ggez", "rs-tiled")
        .window_setup(ggez::conf::WindowSetup::default()
            .title("rs-tiled + ggez example")
            .vsync(false)
        )
        .window_mode(ggez::conf::WindowMode::default()
            .dimensions(1000.0, 800.0)
        )
        // add repo root to ggez filesystem (our example map looks for `assets/tilesheet.png`)
        .add_resource_path(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    let (mut ctx, event_loop) = cb.build()?;

    // construct and start the Game
    let state = Game::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}

struct Game {
    map: MapHandler,
    pan: (f32, f32),
    scale: f32,
}

impl Game {
    fn new(ctx: &mut ggez::Context) -> GameResult<Self> {
        graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);

        // load the map
        let mut cache = FilesystemResourceCache::new();
        let map = Map::parse_file("assets/tiled_base64_external.tmx", &mut cache).unwrap();

        let map_handler = MapHandler::new(map, ctx).unwrap();

        Ok(Self {
            map: map_handler,
            pan: (0.0, 0.0),
            scale: 1.0,
        })
    }
}

impl event::EventHandler<ggez::GameError> for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // fill background color
        let bg_color: ggez::graphics::Color = self.map.background_color().unwrap_or([0.1, 0.2, 0.3, 1.0].into());
        graphics::clear(ctx, bg_color);

        // draw map tiles + objects

        let window_size = graphics::size(ctx);

        let draw_param = DrawParam::default()
            .dest([
                self.pan.0 + window_size.0 / 2.0 - (self.map.width() * self.map.tile_width()) as f32 / 2.0, 
                self.pan.1 + window_size.1 / 2.0 - (self.map.height() * self.map.tile_height()) as f32 / 2.0
                ])
            .scale([self.scale, self.scale]);

        self.map.draw(ctx, draw_param, self.pan)?;

        // draw map bounds

        let rect = self.map.get_bounds();
        let r1 =
            graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::stroke(2.0 / self.scale), rect, graphics::Color::from_rgb_u32(0x888888))?;
        graphics::draw(ctx, &r1, draw_param)?;

        // draw fps

        let fps = ggez::timer::fps(ctx);
        let text = graphics::Text::new(format!("{:.0} fps", fps));

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
        // right click toggles demo animation effect
        if button == MouseButton::Right {
            self.map.example_animate = !self.map.example_animate;
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, _x: f32, _y: f32, dx: f32, dy: f32) {
        // middle click + drag pans the map around
        if input::mouse::button_pressed(ctx, event::MouseButton::Middle) {
            self.pan.0 += dx;
            self.pan.1 += dy;
        }
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        // scroll wheel zooms

        let old_scale = self.scale;
        self.scale *= 1.0 + y as f32 * 0.1;

        // zoom to mouse cursor
        let Point2 { x: mouse_x, y: mouse_y } = input::mouse::position(ctx);
        let (window_width, window_height) = graphics::size(ctx);
        let pos_x = mouse_x - window_width / 2.0 + (self.map.width() * self.map.tile_width()) as f32 / 2.0;
        let pos_y = mouse_y - window_height / 2.0 + (self.map.height() * self.map.tile_height()) as f32 / 2.0;
        self.pan.0 = (self.pan.0 - pos_x) / old_scale * self.scale + pos_x;
        self.pan.1 = (self.pan.1 - pos_y) / old_scale * self.scale + pos_y;
    }
}