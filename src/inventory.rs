use ferrousgl::{GlWindow, Shader, Texture, WindowKey};
use glam::{Mat4, Vec2};
use crate::{ui::ui_quad::UiQuad, voxel::VoxelType};

pub struct Inventory {
    creative_slots: Vec<UiQuad>,  // Holds all voxel types
    slot_selector: UiQuad,
    selected_index: usize,
    ui_scale: f32,
    rows_visible: usize,         // How many rows are visible at once
    current_row: usize,          // Current scroll position
}

impl Inventory {
    pub fn new() -> Self {
        let ui_scale = 1.5;

        let mut slot_selector = UiQuad::new();
        slot_selector.set_size(Vec2::new(30.0 * ui_scale, 30.0 * ui_scale));
        slot_selector.set_position(Vec2::new(15.0 * ui_scale, 15.0 * ui_scale));
        slot_selector.set_uvs([
            Vec2::new((1.0/28.4)*2.0, 1.0/28.4),
            Vec2::new((1.0/28.4)*4.0, 1.0/28.4),
            Vec2::new((1.0/28.4)*4.0, (1.0/28.4)*3.0),
            Vec2::new((1.0/28.4)*2.0, (1.0/28.4)*3.0),
        ]);

        // Initialize creative slots with all voxel types (excluding Air)
        let mut creative_slots = Vec::new();
        let voxel_types = Self::get_all_voxel_types();
        
        for (i, voxel_type) in voxel_types.iter().enumerate() {
            let mut slot = UiQuad::new();
            let x = 15.0 * ui_scale + (i % 10) as f32 * 30.0 * ui_scale;
            let y = 15.0 * ui_scale + (i / 10) as f32 * 30.0 * ui_scale;
            
            slot.set_size(Vec2::new(30.0 * ui_scale, 30.0 * ui_scale));
            slot.set_position(Vec2::new(x, y));
            slot.set_uvs([
                Vec2::new(0.0, 1.0/28.4),
                Vec2::new((1.0/28.4)*2.0, 1.0/28.4),
                Vec2::new((1.0/28.4)*2.0, (1.0/28.4)*3.0),
                Vec2::new(0.0, (1.0/28.4)*3.0),
            ]);
            
            creative_slots.push(slot);
        }

        Self {
            creative_slots,
            selected_index: 0,
            slot_selector,
            ui_scale,
            rows_visible: 1,  // Show one row at a time
            current_row: 0,
        }
    }

    pub fn update(&mut self, window: &GlWindow) -> Option<VoxelType> {
        let voxel_types = Self::get_all_voxel_types();
        
        // Handle mouse scroll to change selection
        let scroll = window.get_mouse_wheel_delta().1;
        if scroll != 0.0 {
            if scroll > 0.0 {
                // Scroll up
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    // Wrap around to end
                    self.selected_index = voxel_types.len() - 1;
                }
            } else {
                // Scroll down
                if self.selected_index < voxel_types.len() - 1 {
                    self.selected_index += 1;
                } else {
                    // Wrap around to start
                    self.selected_index = 0;
                }
            }

            // Adjust the current row based on the selected index
            let new_row = self.selected_index / 10;
            if new_row != self.current_row {
                self.current_row = new_row;
            }

            self.update_slot_selector_position();
            return Some(voxel_types[self.selected_index].clone());
        }
        
        // Handle number key selection for first row (1-0 keys)
        for i in 0..10 {
            let key = match i {
                0 => WindowKey::Num0,
                1 => WindowKey::Num1,
                2 => WindowKey::Num2,
                3 => WindowKey::Num3,
                4 => WindowKey::Num4,
                5 => WindowKey::Num5,
                6 => WindowKey::Num6,
                7 => WindowKey::Num7,
                8 => WindowKey::Num8,
                9 => WindowKey::Num9,
                _ => continue,
            };
            
            if window.is_key_held(key) && i < voxel_types.len() {
                self.selected_index = i;
                self.update_slot_selector_position();
                return Some(voxel_types[i].clone());
            }
        }
        
        None
    }

    fn update_slot_selector_position(&mut self) {
        let x = 15.0 * self.ui_scale + (self.selected_index % 10) as f32 * 30.0 * self.ui_scale;
        let y = 15.0 * self.ui_scale + (self.selected_index / 10) as f32 * 30.0 * self.ui_scale;
        self.slot_selector.set_position(Vec2::new(x, y));
    }

    // Helper function to get all voxel types except Air
    fn get_all_voxel_types() -> Vec<VoxelType> {
        VoxelType::get_all_voxel_types()
    }

    pub fn render(&self, window: &mut GlWindow, shader: &Shader, model: Mat4, projection: Mat4, ui_atlas: &Texture, block_atlas: &Texture) {
        let voxel_types = Self::get_all_voxel_types();
        
        // Determine which slots to render based on current_row
        let start_index = self.current_row * 10;
        let end_index = (self.current_row + self.rows_visible) * 10;
        
        for i in start_index..end_index.min(self.creative_slots.len()) {
            let slot = &self.creative_slots[i];
            
            // Render slot background
            slot.render_with_texture(window, shader, model, projection, Some(ui_atlas));
            
            // Render block texture if available
            if i < voxel_types.len() {
                let voxel_type = &voxel_types[i];
                let face_texture = voxel_type.get_face_texture(0); // Use front face for inventory
                
                if face_texture[0] != -1.0 {
                    let mut block_quad = UiQuad::new();
                    block_quad.set_size(Vec2::new(25.0 * self.ui_scale, 25.0 * self.ui_scale)); // Slightly smaller than slot
                    block_quad.set_position(slot.position);
                    
                    // Set UVs for the block texture
                    block_quad.set_uvs([
                        Vec2::new(face_texture[0], face_texture[1]),
                        Vec2::new(face_texture[0] + face_texture[2], face_texture[1]),
                        Vec2::new(face_texture[0] + face_texture[2], face_texture[1] + face_texture[3]),
                        Vec2::new(face_texture[0], face_texture[1] + face_texture[3]),
                    ]);
                    
                    block_quad.render_with_texture(window, shader, model, projection, Some(block_atlas));
                }
            }
        }
        
        // Render slot selector
        self.slot_selector.render_with_texture(window, shader, model, projection, Some(ui_atlas));
    }

    pub fn get_selected_voxel(&self) -> Option<VoxelType> {
        let voxel_types = Self::get_all_voxel_types();
        if self.selected_index < voxel_types.len() {
            Some(voxel_types[self.selected_index].clone())
        } else {
            None
        }
    }
}
