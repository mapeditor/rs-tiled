//! rs-tiled demo with SFML
//! Displays a map, use WASD keys to move the camera around

mod mesh;
mod tilesheet;

use mesh::VertexMesh;
use sfml::{
    graphics::{BlendMode, Color, Drawable, RenderStates, RenderTarget, RenderWindow, Transform},
    system::{Vector2f, Vector2u},
    window::{ContextSettings, Key, Style},
};
use std::{path::Path, time::Duration};
use tiled::{
    layers::{LayerData, LayerTile},
    map::Map,
};
use tilesheet::Tilesheet;

/// A path to the map to display.
const MAP_PATH: &'static str = "assets/tiled_base64_external.tmx";

/// Wrapper for [LayerTile].
///
/// Wrappers like these are generally recommended to use instead of using the crate structures (e.g. [LayerData]) as you have more freedom
/// with what you can do with them, they won't change between crate versions and they are more specific to your needs.
///
/// [LayerTile]: tiled::layers::LayerTile
/// [LayerData]: tiled::layers::LayerData
struct TileLayer {
    tiles: Vec<LayerTile>,
}

impl TileLayer {
    /// Generates a vertex mesh from this tile layer for rendering.
    pub fn generate_mesh(&self, tilesheet: &Tilesheet, width: u32) -> VertexMesh {
        let height = (self.tiles.len() / width as usize) as u32;
        let mut mesh = VertexMesh::with_capacity((width * height * 4) as usize);
        for x in 0..width {
            for y in 0..height {
                let tile = self.tiles[(x + y * width) as usize];
                if tile.gid != 0 {
                    let uv = tilesheet
                        .tile_uv(tile.gid)
                        .expect(&format!("Could not get tile UV for {:?}", tile));
                    mesh.add_quad(Vector2f::new(x as f32, y as f32), 1., uv);
                }
            }
        }

        mesh
    }
}

/// A [Map] wrapper which also contains graphical information such as the tileset texture or the layer meshes.
///
/// [Map]: tiled::map::Map
pub struct Level {
    layers: Vec<VertexMesh>,
    /// Unique tilesheet related to the level, which contains the Tiled tileset + Its only texture.
    tilesheet: Tilesheet,
}

impl Level {
    /// Create a new level from a Tiled map.
    pub fn from_map(map: Map) -> Self {
        let width = map.width;
        let tilesheet = {
            let tileset = map.tilesets[0].clone();
            Tilesheet::from_tileset(tileset)
        };

        let layers = {
            let layers = map.layers;
            layers
                .iter()
                .map(|layer| TileLayer {
                    tiles: match &layer.tiles {
                        LayerData::Finite(x) => x.iter().copied().collect(),
                        _ => panic!("Infinite map"),
                    },
                })
                .map(|layer| layer.generate_mesh(&tilesheet, width))
                .collect()
        };

        Self { tilesheet, layers }
    }
}

impl Drawable for Level {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut states = states.clone();
        states.set_texture(Some(&self.tilesheet.texture()));
        for mesh in self.layers.iter() {
            target.draw_with_renderstates(mesh, &states);
        }
    }
}

fn main() {
    let map = Map::parse_file(Path::new(MAP_PATH)).unwrap();
    let level = Level::from_map(map);

    let mut window = create_window();

    let mut camera_position = Vector2f::default();

    let mut last_frame_time = std::time::Instant::now();

    loop {
        while let Some(event) = window.poll_event() {
            use sfml::window::Event;
            match event {
                Event::Closed => return,
                _ => (),
            }
        }

        let this_frame_time = std::time::Instant::now();
        let delta_time = this_frame_time - last_frame_time;

        handle_input(&mut camera_position, delta_time);

        let camera_transform = camera_transform(window.size(), camera_position);
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
    context_settings.antialiasing_level = 2;
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

fn camera_transform(window_size: Vector2u, camera_position: Vector2f) -> Transform {
    const TILE_SIZE: f32 = 16.;
    let window_size = Vector2f::new(window_size.x as f32, window_size.y as f32);

    let mut x = Transform::IDENTITY;
    x.scale_with_center(1. / TILE_SIZE, 1. / TILE_SIZE, 0f32, 0f32);
    x.translate(-window_size.x / 2., -window_size.y / 2.);
    x.translate(camera_position.x * TILE_SIZE, camera_position.y * TILE_SIZE);
    x.inverse()
}
