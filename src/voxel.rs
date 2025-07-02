// voxel.rs
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum VoxelType {
    Air,
    Dirt,
    Gravel,
    Sand,
    Sandstone,
    Snow,
    Water,
    Ice,
    Grass,  // Top, Bottom, Sides will be different
    Stone,
    WalnutWood,
    WalnutPlanks,
    WalnutLeaves,
    SpruceWood,
    SprucePlanks,
    SpruceLeaves,
    MahoganyWood,
    MahoganyPlanks,
    MahoganyLeaves,
    Cobblestone,
    CopperOre,
    AmethystOre,
    Stonebrick,
    Glass,
}

impl VoxelType {
    pub fn is_transparent(&self) -> bool {
        matches!(self, VoxelType::Air | VoxelType::Water | VoxelType::Ice | VoxelType::Glass | 
                  VoxelType::WalnutLeaves | VoxelType::SpruceLeaves | VoxelType::MahoganyLeaves)
    }

    pub fn transparent_optimize_outer_only(&self) -> bool {
        matches!(self, VoxelType::Water | VoxelType::Ice)
    }

    pub fn transparent_optimize_outer_one_inner(&self) -> bool {
        matches!(self, VoxelType::Glass | 
                  VoxelType::WalnutLeaves | VoxelType::SpruceLeaves | VoxelType::MahoganyLeaves)
    }

    pub fn is_not_solid(&self) -> bool {
        matches!(self, VoxelType::Air)
    }

    pub fn get_all_voxel_types() -> Vec<VoxelType> {
        VoxelType::iter().filter(|&v| v != VoxelType::Air).collect()
    }

    pub fn get_face_texture(&self, face: usize) -> [f32; 4] {
        // [u, v, tile_width, tile_height]
        // Face order: Top, Bottom, Front, Back, Right, Left
        let tile_w = 1.0 / 16.0;
        let tile_h = 1.0 / 16.0;
        let margin_u = tile_w * 0.01;
        let margin_v = tile_h * 0.01;

        let (u, v) = match self {
            VoxelType::Air => (-1, -1),
            VoxelType::Grass => match face {
                0 => (1, 0),  // Top (grass)
                1 => (0, 0),   // Bottom (dirt)
                _ => (1, 1),   // Sides (grass side)
            },
            VoxelType::Dirt => (0, 0),
            VoxelType::Gravel => (0, 1),
            VoxelType::Sand => (0, 2),
            VoxelType::Sandstone => (2, 6),
            VoxelType::Snow => (0, 3),
            VoxelType::Water => (0, 4),
            VoxelType::Ice => (0, 5),
            VoxelType::Stone => (2, 0),
            VoxelType::WalnutWood => match face {
                0 | 1 => (3, 1),  // Top/Bottom
                _ => (3, 0),      // Sides
            },
            VoxelType::WalnutPlanks => (3, 2),
            VoxelType::WalnutLeaves => (3, 3),
            VoxelType::SpruceWood => match face {
                0 | 1 => (4, 1),  // Top/Bottom
                _ => (4, 0),      // Sides
            },
            VoxelType::SprucePlanks => (4, 2),
            VoxelType::SpruceLeaves => (4, 3),
            VoxelType::MahoganyWood => match face {
                0 | 1 => (5, 1),  // Top/Bottom
                _ => (5, 0),      // Sides
            },
            VoxelType::MahoganyPlanks => (5, 2),
            VoxelType::MahoganyLeaves => (5, 3),
            VoxelType::Cobblestone => (2, 1),
            VoxelType::CopperOre => (2, 2),
            VoxelType::AmethystOre => (2, 3),
            VoxelType::Stonebrick => (2, 4),
            VoxelType::Glass => (2, 5),
        };

        if u == -1 && v == -1 {
            return [-1.0, -1.0, 0.0, 0.0];
        }

        [
            (u + 1) as f32 * tile_w - margin_u,
            (v + 1) as f32 * tile_h - margin_v,
            -tile_w + 2.0 * margin_u,
            -tile_h + 2.0 * margin_v,
        ]
    }

    pub fn item_name(&self) -> &'static str {
        match self {
            VoxelType::Air => "Air",
            VoxelType::Dirt => "Dirt",
            VoxelType::Gravel => "Gravel",
            VoxelType::Sand => "Sand",
            VoxelType::Sandstone => "Sandstone",
            VoxelType::Snow => "Snow",
            VoxelType::Water => "Water",
            VoxelType::Ice => "Ice",
            VoxelType::Grass => "Grass Block",
            VoxelType::Stone => "Stone",
            VoxelType::WalnutWood => "Walnut Wood",
            VoxelType::WalnutPlanks => "Walnut Planks",
            VoxelType::WalnutLeaves => "Walnut Leaves",
            VoxelType::SpruceWood => "Spruce Wood",
            VoxelType::SprucePlanks => "Spruce Planks",
            VoxelType::SpruceLeaves => "Spruce Leaves",
            VoxelType::MahoganyWood => "Mahogany Wood",
            VoxelType::MahoganyPlanks => "Mahogany Planks",
            VoxelType::MahoganyLeaves => "Mahogany Leaves",
            VoxelType::Cobblestone => "Cobblestone",
            VoxelType::CopperOre => "Copper Ore",
            VoxelType::AmethystOre => "Amethyst Ore",
            VoxelType::Stonebrick => "Stone Brick",
            VoxelType::Glass => "Glass",
        }
    }
}