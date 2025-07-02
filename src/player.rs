// player.rs
use ferrousgl::{GlWindow, Mesh, Shader, WindowKey};
use glam::{Mat4, Vec3, Vec4};

use crate::{utils::{collides_aabb::collides_aabb, tree_gen::generate_mahogany_tree}, voxel::VoxelType};

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: (f32, f32), // pitch, yaw
    pub on_ground: bool,
    pub flight_mode: bool,    // New field for flight mode
    pub input_wait_time: f32, // Used to manage input timing
    hand_mesh: Mesh, // Add this line
}

// Player hitbox constants (width, height, depth)
const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_DEPTH: f32 = 0.6;

impl Player {
    pub fn new(position: Vec3) -> Self {

        let hand_mesh = create_cube_mesh();

        Player {
            position,
            velocity: Vec3::ZERO,
            rotation: (0.0, 0.0),
            on_ground: false,
            flight_mode: false, // Default to no flight\
            input_wait_time: 0.0,
            hand_mesh, // Add this line
        }
    }

    pub fn handle_input(
        &mut self,
        window: &mut GlWindow,
        delta_time: f32,
        world: &mut crate::world::World,
        move_mouse: bool,
        selected_voxel: VoxelType,
    ) {
        // Toggle flight mode when F1 is pressed (unchanged)
        if window.is_key_pressed(WindowKey::F1) {
            self.flight_mode = !self.flight_mode;
            if self.flight_mode {
                println!("Flight mode enabled");
            } else {
                println!("Flight mode disabled");
                self.velocity = Vec3::ZERO;
            }
        }

        // --- Mouse look --- (unchanged)
        if move_mouse {
            let mouse_sensitivity = 0.002;
            let mouse_delta = window.get_mouse_delta();
            self.rotation.1 += mouse_delta.0 as f32 * mouse_sensitivity;
            self.rotation.0 -= mouse_delta.1 as f32 * mouse_sensitivity;

            window.set_mouse_position(
                (window.get_window_size().0 as f32 / 2.0) as f64,
                (window.get_window_size().1 as f32 / 2.0) as f64,
            );
        }

        let two_pi = std::f32::consts::TAU;
        self.rotation.1 = (self.rotation.1 + two_pi) % two_pi;
        let pi = std::f32::consts::PI;
        self.rotation.0 = (self.rotation.0 + pi) % (2.0 * pi) - pi;

        self.rotation.1 = (self.rotation.1 + two_pi) % two_pi;
        let pi = std::f32::consts::PI;
        self.rotation.0 = (self.rotation.0 + pi) % (2.0 * pi) - pi;

        // Clamp pitch to avoid looking too far up or down
        let max_pitch = pi / 2.0 - 0.01; // Slightly less than 90 degrees
        if self.rotation.0 > max_pitch {
            self.rotation.0 = max_pitch;
        }
        if self.rotation.0 < -max_pitch {
            self.rotation.0 = -max_pitch;
        }

        // --- Movement ---
        let mut move_dir = Vec3::ZERO;
        let (pitch, yaw) = self.rotation;

        // Calculate movement vectors
        let right = -Vec3::new(yaw.sin(), 0.0, -yaw.cos()).normalize();
        let forward = Vec3::new(right.z, 0.0, -right.x).normalize(); // Perpendicular to forward

        // Handle input
        if window.is_key_held(WindowKey::W) {
            move_dir += forward;
        }
        if window.is_key_held(WindowKey::S) {
            move_dir -= forward;
        }
        if window.is_key_held(WindowKey::D) {
            move_dir += right;
        }
        if window.is_key_held(WindowKey::A) {
            move_dir -= right;
        }

        // Flight mode specific controls
        if self.flight_mode {
            let up = Vec3::Y;
            if window.is_key_held(WindowKey::Space) {
                move_dir += up;
            }
            if window.is_key_held(WindowKey::LeftControl) {
                move_dir -= up;
            }
        }

        // Normalize direction only if we have any input
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        if window.is_key_pressed(WindowKey::Y) {
            // Example voxel data - you'd replace this with your actual data
            let voxels_to_load = generate_mahogany_tree((0, 0, 0));
            
            world.load_voxels_world_async(voxels_to_load, (self.position.x as i32, self.position.y as i32, self.position.z as i32));
        }

        if window.is_mouse_button_pressed(glfw::MouseButtonLeft) {
            if self.input_wait_time == 0.0 {
                if let Some((x, y, z, _, _, _)) = self.raycast(world, 5.0, 0.1) {
                    world.set_voxel_main_thread(x, y, z, crate::voxel::VoxelType::Air);
                    self.input_wait_time = 0.2;
                }
            }
        } else if window.is_mouse_button_pressed(glfw::MouseButtonRight) {
            if self.input_wait_time == 0.0 {
                if let Some((_, _, _, x, y, z)) = self.raycast(world, 5.0, 0.1) {
                    // Place block adjacent to the hit face
                    world.set_voxel_main_thread(x, y, z, selected_voxel); // Or whatever block type you want
                    self.input_wait_time = 0.2;
                }
            }
        } else {
            self.input_wait_time = 0.0;
        }
        self.input_wait_time = (self.input_wait_time - delta_time).max(0.0);

        // --- Movement physics ---
        if self.flight_mode {
            // Flight physics
            let fly_speed = if window.is_key_held(WindowKey::LeftShift) {
                20.0
            } else {
                10.0
            };
            self.velocity = move_dir * fly_speed;
        } else {
            // Ground physics
            let accel = if window.is_key_held(WindowKey::LeftShift) {
                30.0
            } else {
                15.0
            };
            let max_speed = if window.is_key_held(WindowKey::LeftShift) {
                10.0
            } else {
                5.0
            };
            let friction = 10.0;

            // Only apply horizontal movement
            let mut horizontal_velocity = Vec3::new(self.velocity.x, 0.0, self.velocity.z);
            let horizontal_move_dir = Vec3::new(move_dir.x, 0.0, move_dir.z);

            // Accelerate towards move_dir
            let target_vel = horizontal_move_dir * max_speed;
            let vel_change = target_vel - horizontal_velocity;
            let accel_vec = vel_change.clamp_length_max(accel * delta_time);

            horizontal_velocity += accel_vec;

            // Apply friction if no input
            if move_dir.length_squared() == 0.0 {
                horizontal_velocity = horizontal_velocity.lerp(Vec3::ZERO, friction * delta_time);
            }

            self.velocity.x = horizontal_velocity.x;
            self.velocity.z = horizontal_velocity.z;

            // Jumping
            if window.is_key_held(WindowKey::Space) && self.on_ground {
                self.velocity.y = 8.0;
                self.on_ground = false;
            }

            // Gravity
            if !self.on_ground {
                self.velocity.y -= 20.0 * delta_time;
            }
        }

        // --- Collision and movement --- (unchanged)
        let mut new_position = self.position;

        if self.flight_mode {
            new_position += self.velocity * delta_time;
        } else {
            // X axis
            let try_x = new_position + glam::vec3(self.velocity.x * delta_time, 0.0, 0.0);
            if !collides_aabb(try_x, PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_DEPTH, world) {
                new_position.x = try_x.x;
            } else {
                self.velocity.x = 0.0;
            }

            // Y axis
            let try_y = new_position + glam::vec3(0.0, self.velocity.y * delta_time, 0.0);
            if !collides_aabb(try_y, PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_DEPTH, world) {
                new_position.y = try_y.y;
                self.on_ground = false;
            } else {
                if self.velocity.y < 0.0 {
                    self.on_ground = true;
                }
                self.velocity.y = 0.0;
            }

            // Z axis
            let try_z = new_position + glam::vec3(0.0, 0.0, self.velocity.z * delta_time);
            if !collides_aabb(try_z, PLAYER_WIDTH, PLAYER_HEIGHT, PLAYER_DEPTH, world) {
                new_position.z = try_z.z;
            } else {
                self.velocity.z = 0.0;
            }
        }

        self.position = new_position;
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        // Calculate camera position based on player position and rotation
        let (pitch, yaw) = self.rotation;

        // Calculate camera direction
        let direction = Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();

        // Position the camera slightly above the player's head (eye level)
        let eye_position = self.position + Vec3::new(0.0, PLAYER_HEIGHT * 0.9, 0.0);

        // Look slightly above the player's feet for a more natural view
        let target = eye_position + direction;

        // Up vector - typically positive Y, but adjust if you want roll
        let up = Vec3::Y;

        Mat4::look_at_rh(eye_position, target, up)
    }

    pub fn raycast(
        &self,
        world: &crate::world::World,
        max_distance: f32,
        step_size: f32,
    ) -> Option<(i32, i32, i32, i32, i32, i32)> {
        let (pitch, yaw) = self.rotation;

        // Calculate ray direction from player's rotation
        let direction = Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();

        // Start position is player's eye level
        let mut current_pos = self.position + Vec3::new(0.0, PLAYER_HEIGHT * 0.9, 0.0);

        // Convert to block coordinates
        let mut block_x = current_pos.x.floor() as i32;
        let mut block_y = current_pos.y.floor() as i32;
        let mut block_z = current_pos.z.floor() as i32;

        let mut last_block_x = block_x;
        let mut last_block_y = block_y;
        let mut last_block_z = block_z;

        let mut distance = 0.0;

        while distance < max_distance {
            // Check current block
            if world.get_voxel(block_x, block_y, block_z) != crate::voxel::VoxelType::Air {
                return Some((
                    block_x,
                    block_y,
                    block_z,
                    last_block_x,
                    last_block_y,
                    last_block_z,
                ));
            }

            // Save last solid block position
            last_block_x = block_x;
            last_block_y = block_y;
            last_block_z = block_z;

            // Advance ray
            current_pos += direction * step_size;
            distance += step_size;

            // Update block coordinates
            block_x = current_pos.x.floor() as i32;
            block_y = current_pos.y.floor() as i32;
            block_z = current_pos.z.floor() as i32;
        }

        None
    }
}

fn create_cube_mesh() -> Mesh {
    // 8 unique vertices (position, normal, uv)
    let vertices: [f32; 8 * (3 + 3 + 2)] = [
        // Position (3)       Normal (3)       UV (2)
        // Front bottom left
        -0.5, -0.5,  0.5,   0.0,  0.0,  1.0,   0.0, 0.0,
        // Front bottom right
         0.5, -0.5,  0.5,   0.0,  0.0,  1.0,   1.0, 0.0,
        // Front top right
         0.5,  0.5,  0.5,   0.0,  0.0,  1.0,   1.0, 1.0,
        // Front top left
        -0.5,  0.5,  0.5,   0.0,  0.0,  1.0,   0.0, 1.0,
        // Back bottom left
        -0.5, -0.5, -0.5,   0.0,  0.0, -1.0,   1.0, 0.0,
        // Back bottom right
         0.5, -0.5, -0.5,   0.0,  0.0, -1.0,   0.0, 0.0,
        // Back top right
         0.5,  0.5, -0.5,   0.0,  0.0, -1.0,   0.0, 1.0,
        // Back top left
        -0.5,  0.5, -0.5,   0.0,  0.0, -1.0,   1.0, 1.0,
    ];

    // 36 indices (6 faces * 2 triangles * 3 vertices)
    let indices: [u32; 36] = [
        // Front face
        0, 1, 2, 2, 3, 0,
        // Back face
        5, 4, 7, 7, 6, 5,
        // Top face
        3, 2, 6, 6, 7, 3,
        // Bottom face
        4, 5, 1, 1, 0, 4,
        // Right face
        1, 5, 6, 6, 2, 1,
        // Left face
        4, 0, 3, 3, 7, 4,
    ];

    let mut mesh = Mesh::new();
    mesh.update_vertices(&vertices);
    // Add vertex attributes: position (3), normal (3), uv (2)
    mesh.add_vertex_attributes(&[
        (0, 3, gl::FLOAT, false),  // position
        (1, 3, gl::FLOAT, false),  // normal
        (2, 2, gl::FLOAT, false),  // uv
    ]);
    mesh.update_indices(&indices);
    mesh
}