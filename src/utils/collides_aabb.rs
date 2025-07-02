use crate::voxel::VoxelType;

// Robust AABB collision with voxels for a given hitbox size
pub fn collides_aabb(
    pos: glam::Vec3,
    width: f32,
    height: f32,
    depth: f32,
    world: &crate::world::World,
) -> bool {
    let min = glam::vec3(
        pos.x - width / 2.0,
        pos.y,
        pos.z - depth / 2.0,
    );
    let max = glam::vec3(
        pos.x + width / 2.0,
        pos.y + height,
        pos.z + depth / 2.0,
    );
    for x in min.x.floor() as i32..max.x.ceil() as i32 {
        for y in min.y.floor() as i32..max.y.ceil() as i32 {
            for z in min.z.floor() as i32..max.z.ceil() as i32 {
                if !world.get_voxel(x, y, z).is_not_solid() {
                    return true;
                }
            }
        }
    }
    false
}