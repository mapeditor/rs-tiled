use std::path::Path;

use sfml::{
    graphics::{FloatRect, IntRect, Texture},
    SfBox,
};
use tiled::tileset::Tileset;

/// A container for a tileset and the texture it references.
pub struct Tilesheet {
    texture: SfBox<Texture>,
    tileset: Tileset,
}

impl Tilesheet {
    /// Create a tilesheet from a Tiled tileset, loading its texture along the way.
    pub fn from_tileset<'p>(tileset: Tileset) -> Self {
        let tileset_image = tileset.image.as_ref().unwrap();

        let texture = {
            let texture_path = Path::new(&tileset_image.source);

            Texture::from_file(texture_path.to_str().expect("obtaining valid UTF-8 path")).unwrap()
        };

        Tilesheet { texture, tileset }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn tile_rect(&self, gid: u32) -> Option<IntRect> {
        if gid == 0 {
            return None;
        }
        let id = gid - self.tileset.first_gid;

        let tile_width = self.tileset.tile_width;
        let tile_height = self.tileset.tile_height;
        let tiles_per_row = self.texture.size().x / tile_width;
        let x = id % tiles_per_row * tile_width;
        let y = id / tiles_per_row * tile_height;

        Some(IntRect {
            left: x as i32,
            top: y as i32,
            width: tile_width as i32,
            height: tile_height as i32,
        })
    }

    pub fn tile_uv(&self, gid: u32) -> Option<FloatRect> {
        if let Some(IntRect {
            left,
            top,
            width,
            height,
        }) = self.tile_rect(gid)
        {
            // In SFML, UVs are in pixel coordinates, so we just grab the tile rect and convert it
            // into a FloatRect
            Some(FloatRect {
                left: left as f32,
                top: top as f32,
                width: width as f32,
                height: height as f32,
            })
        } else {
            None
        }
    }
}
