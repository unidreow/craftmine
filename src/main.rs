// main.rs
use ferrousgl::{DepthType, GlWindow, Mesh, RenderTexture, Shader, Texture, WindowConfig, WindowKey};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rand::{rng, Rng};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

mod voxel;
mod chunk;
mod player;
mod utils;
mod ui;
mod inventory;
mod world;

use voxel::VoxelType;
use chunk::Chunk;
use player::Player;
use world::World;
use crate::inventory::Inventory;
use crate::ui::ui_quad::UiQuad;

fn main() {
    // Show a message box with panic info if an error occurs
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic"
        };
        let loc = info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        #[cfg(windows)]
        {
            use winapi::um::winuser::{MessageBoxW, MB_OK, MB_ICONERROR};
            use std::os::windows::ffi::OsStrExt;
            use std::ffi::OsStr;
            let text = format!("Rust Backend panicked at: {}\n\n{}", loc, msg);
            let wide: Vec<u16> = OsStr::new(&text).encode_wide().chain(Some(0)).collect();
            let title: Vec<u16> = OsStr::new("Error").encode_wide().chain(Some(0)).collect();
            unsafe { MessageBoxW(std::ptr::null_mut(), wide.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR); }
        }
        #[cfg(not(windows))]
        eprintln!("Panic at {}\n{}", loc, msg);
    }));

    let mut rng = rand::rng();

    // Initialize window
    let config = WindowConfig {
        width: 1280,
        height: 720,
        anti_aliasing: 4,
        target_framerate: 1000,
        title: "Craftmine - Waiting for FGL".to_string(),
        ..Default::default()
    };
    let mut window = GlWindow::new(config);

    // Load shaders
    let shader = Shader::new_from_file(
        Path::new("assets/shaders/voxel.vert"),
        Path::new("assets/shaders/voxel.frag"),
    ).expect("Failed to load shaders");

    // Load texture atlas
    let atlas = Texture::new_from_file(Path::new("assets/textures/atlas.png"))
        .expect("Failed to load texture atlas");
    atlas.bind(0);
    atlas.set_mipmap_and_filtering(ferrousgl::MipmapType::Nearest, ferrousgl::FilterMode::Nearest);
    atlas.unbind();

    let atlas_normal = Texture::new_from_file(Path::new("assets/textures/atlas_normal.png"))
        .expect("Failed to load texture atlas");
    atlas_normal.bind(0);
    atlas_normal.set_mipmap_and_filtering(ferrousgl::MipmapType::Nearest, ferrousgl::FilterMode::Nearest);
    atlas_normal.unbind();

    let ui_tex = Texture::new_from_file(Path::new("assets/textures/ui.png"))
        .expect("Failed to load texture ui atlas");
    ui_tex.bind(0);
    ui_tex.set_mipmap_and_filtering(ferrousgl::MipmapType::Nearest, ferrousgl::FilterMode::Nearest);
    ui_tex.unbind();

    // Create player
    let mut player = Player::new(Vec3::new(0.0, 300.0, 0.0));

    // Create world
    let mut rng = rand::rng();
    let mut world = World::new(64, rng.random_range(0..100000));
    println!("Seed: {}", world.seed);

    // Generate initial world
    //world.generate_chunks_around_center(0, 0, 0, 6);

    // FPS counter variables
    let mut last_fps_update = Instant::now();
    let mut frame_count = 0;
    let mut fps = 0;

    // Timing for delta_time
    let mut last_frame_time = Instant::now();

    // Load UI shaders
    let ui_shader = Shader::new_from_file(
        Path::new("assets/shaders/ui.vert"),
        Path::new("assets/shaders/ui.frag"),
    ).expect("Failed to load UI shaders");

    // Create UI quad
    let mut crosshair = UiQuad::new();
    crosshair.set_size(Vec2::new(25.0, 25.0)); // Set size of the UI quad
    crosshair.set_uvs([
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0/28.4, 0.0),
        Vec2::new(1.0/28.4, 1.0/28.4),
        Vec2::new(0.0, 1.0/28.4),
    ]);
    
    let mut inventory = Inventory::new();

    // Main game loop
    while !window.should_window_close() {

        // Calculate delta_time
        let now = Instant::now();
        let delta_time = now.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = now;

        window.clear_color(Vec4::new(0.77, 0.87, 1.0, 1.0)); // Clear with sky color
        window.clear_depth(); // Clear depth buffer
        window.set_depth_testing(DepthType::LessOrEqual);

        // Handle input
        player.handle_input(&mut window, delta_time, &mut world, true, inventory.get_selected_voxel().unwrap());
        inventory.update(&window);

        // Set up view/projection matrices
        let projection = Mat4::perspective_rh_gl(
            70.0f32.to_radians(),
            window.get_window_size().0 as f32 / window.get_window_size().1 as f32,
            0.1,
            1000.0,
        );
        let view = player.get_view_matrix();

        // Render world
        shader.bind_program();
        atlas.bind(0); // Bind to texture unit 0
        atlas_normal.bind(1);
        shader.set_uniform_texture("atlas", 0); // Set sampler uniform
        shader.set_uniform_texture("normalMap", 1);
        shader.set_uniform_matrix_4fv("projection", projection.as_ref());
        shader.set_uniform_matrix_4fv("view", view.as_ref());
        shader.set_uniform_3f("viewPos", player.position.x, player.position.y, player.position.z);

        world.render(&window, &shader);
        shader.unbind_program();

        world.process_chunk_updates(&window);
        // Convert player position to chunk coordinates
        

        //world.update_chunks_around_player(chunk_x, chunk_y, chunk_z);

        if window.is_key_held(WindowKey::F3) {
            let chunk_x = (player.position.x / world.chunk_size as f32).floor() as i32;
            let chunk_y = (player.position.y / world.chunk_size as f32).floor() as i32;
            let chunk_z = (player.position.z / world.chunk_size as f32).floor() as i32;

            world.generate_nearest_missing_chunk_simple(chunk_x, chunk_y, chunk_z);
        }


        // --- Render UI quad ---
        let (w, h) = window.get_window_size();
        let ortho = Mat4::orthographic_rh_gl(0.0, w as f32, h as f32, 0.0, -1.0, 1.0);
        let model = Mat4::from_scale(Vec3::new(1.0, 1.0, 1.0));
        crosshair.render_with_texture(&mut window, &ui_shader, Mat4::from_translation(Vec3::new(w as f32/2.0, h as f32/2.0, 0.0))
            * model, ortho, Some(&ui_tex));

        inventory.render(&mut window, &ui_shader, model, ortho, &ui_tex, &atlas);
        
        // Update window
        window.update();

        // FPS counting
        frame_count += 1;
        if last_fps_update.elapsed().as_secs_f32() >= 1.0 {
            fps = frame_count;
            frame_count = 0;
            last_fps_update = Instant::now();
            // Update window title with FPS
            window.set_window_title(
                &format!(
                    "Craftmine (fps: {})", 
                    fps
                )
            );
        }
    }
}