use rand::Rng;
use crate::{utils::interpolate::lerp, voxel::VoxelType};
use std::f64::consts::PI;

/// Generic tree generator for different wood/leaf types
fn generate_tree_generic(
    base_pos: (i32, i32, i32),
    wood: VoxelType,
    leaves: VoxelType,
    trunk_height_range: std::ops::Range<i32>,
    branch_length_range: std::ops::Range<i32>,
    leaf_radius_range: std::ops::Range<i32>,
    crown_radius_range: std::ops::Range<i32>,
    trunk_thickness: i32, // <-- new parameter
) -> Vec<(i32, i32, i32, VoxelType)> {
    let (x, y, z) = base_pos;
    let mut voxels = Vec::new();
    let mut rng = rand::rng();

    // Generate trunk
    let trunk_height = rng.random_range(trunk_height_range.clone());
    let trunk_top = y + trunk_height;

    // Create trunk (thickness x thickness blocks)
    for dx in 0..trunk_thickness {
        for dz in 0..trunk_thickness {
            for dy in 0..trunk_height {
                voxels.push((x + dx, y + dy, z + dz, wood));

                // Random chance to spawn a branch
                if dy > 5 && dy < trunk_height - 5 && rng.random_bool(0.2) {
                    let start_point = (x + dx, y + dy, z + dz);

                    // Branch parameters
                    let max_length = rng.random_range(branch_length_range.clone());
                    let y_offset = rng.random_range(1..4);
                    let x_offset = rng.random_range(-max_length..max_length);
                    let z_offset = rng.random_range(-max_length..max_length);

                    let end_point = (
                        start_point.0 + x_offset,
                        start_point.1 + y_offset,
                        start_point.2 + z_offset,
                    );

                    // Draw branch
                    let steps = max_length * 2;
                    for i in 0..=steps {
                        let t = i as f32 / steps as f32;
                        let bx = lerp(start_point.0 as f32, end_point.0 as f32, t).round() as i32;
                        let by = lerp(start_point.1 as f32, end_point.1 as f32, t).round() as i32;
                        let bz = lerp(start_point.2 as f32, end_point.2 as f32, t).round() as i32;
                        voxels.push((bx, by, bz, wood));
                    }

                    // Add leaves at branch end
                    generate_leaves_custom(&mut voxels, end_point, leaves, leaf_radius_range.clone());
                }
            }
        }
    }

    // Add leaves at the top of the tree
    generate_leaves_crown_custom(&mut voxels, (x, trunk_top, z), leaves, crown_radius_range);

    voxels
}

/// Spruce tree generator
pub fn generate_spruce_tree(base_pos: (i32, i32, i32)) -> Vec<(i32, i32, i32, VoxelType)> {
    generate_tree_generic(
        base_pos,
        VoxelType::SpruceWood,
        VoxelType::SpruceLeaves,
        14..22,      // Spruce trunk height
        3..6,        // Spruce branch length
        1..3,        // Spruce leaf cluster radius
        2..4,        // Spruce crown radius
        1,           // Spruce trunk thickness
    )
}

/// Mahogany tree generator
pub fn generate_mahogany_tree(base_pos: (i32, i32, i32)) -> Vec<(i32, i32, i32, VoxelType)> {
    let mut rng = rand::rng();
    let thickness = rng.random_range(2..3); // 2 or 3
    generate_tree_generic(
        base_pos,
        VoxelType::MahoganyWood,
        VoxelType::MahoganyLeaves,
        18..32,      // Mahogany trunk height
        7..12,        // Mahogany branch length
        2..4,        // Mahogany leaf cluster radius
        3..5,        // Mahogany crown radius
        thickness,   // Mahogany trunk thickness (2 or 3)
    )
}

/// Walnut tree generator
pub fn generate_walnut_tree(base_pos: (i32, i32, i32)) -> Vec<(i32, i32, i32, VoxelType)> {
    generate_tree_generic(
        base_pos,
        VoxelType::WalnutWood,
        VoxelType::WalnutLeaves,
        3..7,     // Walnut trunk height
        2..5,       // Walnut branch length
        1..3,       // Walnut leaf cluster radius
        2..4,       // Walnut crown radius
        2,          // Walnut trunk thickness
    )
}

// Helper for leaves with custom type and radius
fn generate_leaves_custom(
    voxels: &mut Vec<(i32, i32, i32, VoxelType)>,
    center: (i32, i32, i32),
    leaf_type: VoxelType,
    radius_range: std::ops::Range<i32>,
) {
    let (cx, cy, cz) = center;
    let mut rng = rand::rng();
    let radius = rng.random_range(radius_range);

    for dx in -radius..=radius {
        for dy in -1..=1 {
            for dz in -radius..=radius {
                let distance_sq = dx * dx + dy * dy + dz * dz;
                if distance_sq <= radius * radius {
                    let x = cx + dx;
                    let y = cy + dy;
                    let z = cz + dz;

                    if !voxels.iter().any(|&(vx, vy, vz, _)| vx == x && vy == y && vz == z) {
                        voxels.push((x, y, z, leaf_type));
                    }
                }
            }
        }
    }
}

// Helper for crown with custom type and radius
fn generate_leaves_crown_custom(
    voxels: &mut Vec<(i32, i32, i32, VoxelType)>,
    center: (i32, i32, i32),
    leaf_type: VoxelType,
    radius_range: std::ops::Range<i32>,
) {
    let (cx, cy, cz) = center;
    let mut rng = rand::rng();
    let radius = rng.random_range(radius_range);

    for dx in -radius..=radius {
        for dy in 0..radius {
            for dz in -radius..=radius {
                let x_norm = dx as f32 / radius as f32;
                let y_norm = dy as f32 / (radius as f32 * 0.8);
                let z_norm = dz as f32 / radius as f32;
                let distance_sq = x_norm * x_norm + y_norm * y_norm + z_norm * z_norm;

                if distance_sq <= 1.0 {
                    let x = cx + dx;
                    let y = cy + dy;
                    let z = cz + dz;

                    if !voxels.iter().any(|&(vx, vy, vz, _)| vx == x && vy == y && vz == z) {
                        voxels.push((x, y, z, leaf_type));
                    }
                }
            }
        }
    }
}