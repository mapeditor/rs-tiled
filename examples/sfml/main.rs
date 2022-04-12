//! ## rs-tiled demo with SFML
//! --------------------------
//! Displays a map, use WASD keys to move the camera around.
//! Only draws its tile layers and nothing else.

mod mesh;
mod tilesheet;

use mesh::QuadMesh;
use sfml::{
    graphics::{BlendMode, Color, Drawable, RenderStates, RenderTarget, RenderWindow, Transform},
    system::{Vector2f, Vector2u},
    window::{ContextSettings, Key, Style},
};
use std::{env, path::PathBuf, time::Duration};
use tiled::{FiniteTileLayer, Loader, Map};
use tilesheet::Tilesheet;

/// A path to the map to display.
const MAP_PATH: &str = "assets/tiled_base64_external.tmx";

/// A [Map] wrapper which also contains graphical information such as the tileset texture or the layer meshes.
///
/// Wrappers like these are generally recommended to use instead of using the crate structures (e.g. [LayerData]) as you have more freedom
/// with what you can do with them, they won't change between crate versions and they are more specific to your needs.
///
/// [Map]: tiled::map::Map
pub struct Level {
    layers: Vec<QuadMesh>,
    /// Unique tilesheet related to the level, which contains the Tiled tileset + Its only texture.
    tilesheet: Tilesheet,
    tile_size: f32,
}

impl Level {
    /// Create a new level from a Tiled map.
    pub fn from_map(map: Map) -> Self {
        let tilesheet = {
            let tileset = map.tilesets()[0].clone();
            Tilesheet::from_tileset(tileset)
        };
        let tile_size = map.tile_width as f32;

        let layers = map
            .layers()
            .filter_map(|layer| match &layer.layer_type() {
                tiled::LayerType::Tiles(l) => Some(generate_mesh(
                    match l {
                        tiled::TileLayer::Finite(f) => f,
                        tiled::TileLayer::Infinite(_) => panic!("Infinite maps not supported"),
                    },
                    &tilesheet,
                )),
                _ => None,
            })
            .collect();

        Self {
            tilesheet,
            layers,
            tile_size,
        }
    }
}

/// Generates a vertex mesh from a tile layer for rendering.
fn generate_mesh(layer: &FiniteTileLayer, tilesheet: &Tilesheet) -> QuadMesh {
    let (width, height) = (layer.width() as usize, layer.height() as usize);
    let mut mesh = QuadMesh::with_capacity(width * height);
    for x in 0..width as i32 {
        for y in 0..height as i32 {
            if let Some(tile) = layer.get_tile(x, y) {
                let uv = tilesheet.tile_rect(tile.id());
                mesh.add_quad(Vector2f::new(x as f32, y as f32), 1., uv);
            }
        }
    }

    mesh
}

impl Drawable for Level {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut states = *states;
        states.set_texture(Some(self.tilesheet.texture()));
        for mesh in self.layers.iter() {
            target.draw_with_renderstates(mesh, &states);
        }
    }
}

fn main() {
    let mut loader = Loader::new();

    let map = loader
        .load_tmx_map(
            PathBuf::from(
                env::var("CARGO_MANIFEST_DIR")
                    .expect("To run the example, use `cargo run --example sfml`"),
            )
            .join(MAP_PATH),
        )
        .unwrap();
    let level = Level::from_map(map);

    let mut window = create_window();
    let mut camera_position = Vector2f::default();
    let mut last_frame_time = std::time::Instant::now();

    loop {
        while let Some(event) = window.poll_event() {
            use sfml::window::Event;
            if event == Event::Closed {
                return;
            }
        }

        let this_frame_time = std::time::Instant::now();
        let delta_time = this_frame_time - last_frame_time;

        handle_input(&mut camera_position, delta_time);

        let camera_transform = camera_transform(window.size(), camera_position, level.tile_size);
        let render_states = RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

        window.clear(Color::BLACK);
        window.draw_with_renderstates(&level, &render_states);
        window.display();

        last_frame_time = this_frame_time;
    }
}

/// Creates the window of the application
fn create_window() -> RenderWindow {
    let mut context_settings = ContextSettings::default();
    context_settings.set_antialiasing_level(2);
    let mut window = RenderWindow::new(
        (1080, 720),
        "rs-tiled demo",
        Style::CLOSE,
        &context_settings,
    );
    window.set_vertical_sync_enabled(true);

    window
}

fn handle_input(camera_position: &mut Vector2f, delta_time: Duration) {
    let mut movement = Vector2f::default();

    const SPEED: f32 = 5.;
    if Key::W.is_pressed() {
        movement.y -= 1.;
    }
    if Key::A.is_pressed() {
        movement.x -= 1.;
    }
    if Key::S.is_pressed() {
        movement.y += 1.;
    }
    if Key::D.is_pressed() {
        movement.x += 1.;
    }

    *camera_position += movement * delta_time.as_secs_f32() * SPEED;
}

fn camera_transform(window_size: Vector2u, camera_position: Vector2f, tile_size: f32) -> Transform {
    let window_size = Vector2f::new(window_size.x as f32, window_size.y as f32);

    let mut x = Transform::IDENTITY;
    x.translate(window_size.x / 2., window_size.y / 2.);
    x.translate(
        -camera_position.x * tile_size,
        -camera_position.y * tile_size,
    );
    x.scale_with_center(tile_size, tile_size, 0f32, 0f32);
    x
}
