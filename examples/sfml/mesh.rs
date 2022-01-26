use sfml::{
    graphics::{Drawable, FloatRect, PrimitiveType, Vertex},
    system::Vector2f,
};

pub struct QuadMesh(Vec<Vertex>);

impl QuadMesh {
    /// Create a new mesh with capacity for the given amount of vertices.
    pub fn with_capacity(quads: usize) -> Self {
        Self(Vec::with_capacity(quads * 4))
    }

    /// Add a quad made up of vertices to the mesh.
    pub fn add_quad(&mut self, position: Vector2f, size: f32, uv: FloatRect) {
        self.0.push(Vertex::with_pos_coords(
            position,
            Vector2f::new(uv.left, uv.top),
        ));
        self.0.push(Vertex::with_pos_coords(
            position + Vector2f::new(size, 0f32),
            Vector2f::new(uv.left + uv.width, uv.top),
        ));
        self.0.push(Vertex::with_pos_coords(
            position + Vector2f::new(size, size),
            Vector2f::new(uv.left + uv.width, uv.top + uv.height),
        ));
        self.0.push(Vertex::with_pos_coords(
            position + Vector2f::new(0f32, size),
            Vector2f::new(uv.left, uv.top + uv.height),
        ));
    }
}

impl Drawable for QuadMesh {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw_primitives(&self.0, PrimitiveType::QUADS, states);
    }
}
