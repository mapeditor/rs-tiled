extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate tiled;

use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;
use std::fs::File;

use sdl2_window::Sdl2Window;
use opengl_graphics::{
    OpenGL,
    Texture,
};
use graphics::{Image, Graphics, default_draw_state};
use graphics::math::Matrix2d;
use piston::event::*;

fn main() {
    let (width, height) = (500, 500);
    let opengl = OpenGL::_3_2;
    let window = Sdl2Window::new(
        opengl,
        piston::window::WindowSettings::new(
            "Tiled + Piston Example".to_string(),
            piston::window::Size {width: width, height: height}
        ).exit_on_esc(true)
    );
    let window = Rc::new(RefCell::new(window));
    let ref mut gl = opengl_graphics::GlGraphics::new(opengl);
    let mut blank_image = Image::new();

    let tmx_file = File::open(&Path::new("assets/tiled_base64_zlib.tmx")).ok().expect("could not open tmx");
    let tmx_map = tiled::parse(tmx_file).unwrap();
    let image_path = Path::new("assets/tilesheet.png");
    let image_tex = Texture::from_path(&image_path).unwrap();

    for e in window.events() {
        use piston::event::{RenderEvent};

        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                graphics::clear([0.0, 0.0, 0.0, 1.0], gl);
                render_tiled_map(&tmx_map, &image_tex, &mut blank_image, c.transform, gl);
            });
        }
    }
}

fn render_tiled_map<G>(map: &tiled::Map,
                       image_tex: &<G as Graphics>::Texture,
                       drawn_image: &mut Image,
                       transform: Matrix2d,
                       g: &mut G)
    where G: Graphics
{
    let ref tile_set = map.tilesets[0];
    let (tile_width, tile_height) = (tile_set.tile_width, tile_set.tile_height);
    let tiles_in_row = (tile_set.images[0].width + tile_set.spacing as i32 )/ (tile_width as i32 + tile_set.spacing as i32);

    for layer in map.layers.iter() {
        for (i, tile_row) in layer.tiles.iter().enumerate() {
            for (j, tile_val) in tile_row.iter().enumerate() {
                if *tile_val == 0 {
                    continue;
                }
                let tile_x = *tile_val % tiles_in_row as u32 - 1;
                let tile_y = *tile_val / tiles_in_row as u32;

                let pixel_x = ((tile_x) * tile_width) + ((tile_x) * tile_set.spacing);
                let pixel_y = ((tile_y) * tile_height) + ((tile_y) * tile_set.spacing);

                drawn_image.src_rect([pixel_x as i32, pixel_y as i32, tile_width as i32, tile_height as i32])
                           .rect([j as f64 * tile_width as f64, i as f64 * tile_height as f64,
                                  tile_width as f64, tile_height as f64])
                           .draw(image_tex, default_draw_state(), transform, g);
            }
        }
    }
}
