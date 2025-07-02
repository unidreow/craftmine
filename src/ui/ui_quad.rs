use ferrousgl::{Mesh, GlWindow, Shader, Texture};
use glam::{Mat4, Vec2, Vec3};

pub struct UiQuad {
    mesh: Mesh,
    pub size: Vec2,
    pub position: Vec2,
    uvs: [Vec2; 4], // Removed texture field
}

impl UiQuad {
    pub fn new() -> Self {
        let quad_vertices: [f32; 20] = [
            // positions        // texcoords
            -0.5, -0.5, 0.0,    0.0, 0.0,
             0.5, -0.5, 0.0,    1.0, 0.0,
             0.5,  0.5, 0.0,    1.0, 1.0,
            -0.5,  0.5, 0.0,    0.0, 1.0,
        ];
        let quad_indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
        let mut mesh = Mesh::new();
        mesh.update_vertices(&quad_vertices);
        mesh.update_indices(&quad_indices);
        mesh.add_vertex_attributes(&[
            (0, 3, gl::FLOAT, false),  // position (x, y, z)
            (1, 2, gl::FLOAT, false)   // texcoord (u, v)
        ]);
        Self {
            mesh,
            size: Vec2::ONE,
            position: Vec2::ZERO,
            uvs: [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ],
        }
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn set_pixel_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn set_uvs(&mut self, uvs: [Vec2; 4]) {
        self.uvs = uvs;
        // Update mesh vertices with new UVs
        let quad_vertices: [f32; 20] = [
            // positions        // texcoords
            -0.5, -0.5, 0.0,    self.uvs[0].x, self.uvs[0].y,
             0.5, -0.5, 0.0,    self.uvs[1].x, self.uvs[1].y,
             0.5,  0.5, 0.0,    self.uvs[2].x, self.uvs[2].y,
            -0.5,  0.5, 0.0,    self.uvs[3].x, self.uvs[3].y,
        ];
        self.mesh.update_vertices(&quad_vertices);
    }

    pub fn render(&self, window: &mut GlWindow, shader: &Shader, model: Mat4, projection: Mat4) {
        shader.bind_program();
        shader.set_uniform_matrix_4fv("projection", projection.as_ref());

        // Model: translate to pixel position, then scale to pixel size
        let translate = Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0));
        let scale = Mat4::from_scale(Vec3::new(self.size.x, self.size.y, 1.0));
        let final_model = model * translate * scale;
        shader.set_uniform_matrix_4fv("model", final_model.as_ref());

        window.render_mesh(&self.mesh);
        shader.unbind_program();
    }

    // Add a render_with_texture method
    pub fn render_with_texture(
        &self,
        window: &mut GlWindow,
        shader: &Shader,
        model: Mat4,
        projection: Mat4,
        texture: Option<&Texture>,
    ) {
        shader.bind_program();
        shader.set_uniform_matrix_4fv("projection", projection.as_ref());

        let translate = Mat4::from_translation(Vec3::new(self.position.x, self.position.y, 0.0));
        let scale = Mat4::from_scale(Vec3::new(self.size.x, self.size.y, 1.0));
        let final_model = model * translate * scale;
        shader.set_uniform_matrix_4fv("model", final_model.as_ref());

        if let Some(texture) = texture {
            texture.bind(0);
            shader.set_uniform_1i("tex", 0);
        }

        window.render_mesh(&self.mesh);
        shader.unbind_program();
    }
}