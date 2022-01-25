use sfml::{
    graphics::{Color, Drawable, FloatRect, PrimitiveType, Vertex},
    system::Vector2f,
};

pub struct VertexMesh(Vec<Vertex>);

impl VertexMesh {
    /// Create a new mesh with capacity for the given amount of vertices.
    pub fn with_capacity(vertices: usize) -> Self {
        Self(Vec::with_capacity(vertices))
    }

    /// Add a quad made up of vertices to the mesh.
    pub fn add_quad(&mut self, position: Vector2f, size: f32, uv: FloatRect) {
        self.0.push(Vertex::new(
            position,
            Color::WHITE,
            Vector2f::new(uv.left, uv.top),
        ));
        self.0.push(Vertex::new(
            position + Vector2f::new(size, 0f32),
            Color::WHITE,
            Vector2f::new(uv.left + uv.width, uv.top),
        ));
        self.0.push(Vertex::new(
            position + Vector2f::new(size, size),
            Color::WHITE,
            Vector2f::new(uv.left + uv.width, uv.top + uv.height),
        ));
        self.0.push(Vertex::new(
            position + Vector2f::new(0f32, size),
            Color::WHITE,
            Vector2f::new(uv.left, uv.top + uv.height),
        ));
    }
}

impl Drawable for VertexMesh {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw_primitives(&self.0, PrimitiveType::QUADS, states);
    }
}
