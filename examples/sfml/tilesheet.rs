use std::sync::Arc;

use sfml::{
    graphics::{FloatRect, Texture},
    SfBox,
};
use tiled::Tileset;

/// A container for a tileset and the texture it references.
pub struct Tilesheet {
    texture: SfBox<Texture>,
    tileset: Arc<Tileset>,
}

impl Tilesheet {
    /// Create a tilesheet from a Tiled tileset, loading its texture along the way.
    pub fn from_tileset<'p>(tileset: Arc<Tileset>) -> Self {
        let tileset_image = tileset.image.as_ref().unwrap();

        let texture = {
            let texture_path = &tileset_image
                .source
                .to_str()
                .expect("obtaining valid UTF-8 path");
            Texture::from_file(texture_path).unwrap()
        };

        Tilesheet { texture, tileset }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn tile_rect(&self, id: u32) -> FloatRect {
        let tile_width = self.tileset.tile_width;
        let tile_height = self.tileset.tile_height;
        let spacing = self.tileset.spacing;
        let margin = self.tileset.margin;
        let tiles_per_row = (self.texture.size().x - margin + spacing) / (tile_width + spacing);
        let x = id % tiles_per_row * tile_width;
        let y = id / tiles_per_row * tile_height;

        FloatRect {
            left: x as f32,
            top: y as f32,
            width: tile_width as f32,
            height: tile_height as f32,
        }
    }
}
