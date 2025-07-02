pub fn cube_face(
    x: usize,
    y: usize,
    z: usize,
    uvs: [f32; 4],
    face: usize,
) -> (Vec<[f32; 4]>, Vec<[f32; 2]>) {  // Changed to [f32; 4] for positions
    let x = x as f32;
    let y = y as f32;
    let z = z as f32;
    let (u, v, tile_w, tile_h) = (uvs[0], uvs[1], uvs[2], uvs[3]);
    
    // Default AO value (will be modified per-vertex later)
    let default_ao = 1.0;
    
    match face {
        0 => {
            // Top (+Y)
            (
                vec![
                    [x, y + 1.0, z + 1.0, default_ao],
                    [x + 1.0, y + 1.0, z + 1.0, default_ao],
                    [x + 1.0, y + 1.0, z, default_ao],
                    [x, y + 1.0, z, default_ao],
                ],
                vec![
                    [u, v + tile_h],
                    [u + tile_w, v + tile_h],
                    [u + tile_w, v],
                    [u, v],
                ],
            )
        }
        1 => {
            // Bottom (-Y)
            (
                vec![
                    [x, y, z, default_ao],
                    [x + 1.0, y, z, default_ao],
                    [x + 1.0, y, z + 1.0, default_ao],
                    [x, y, z + 1.0, default_ao],
                ],
                vec![
                    [u, v],
                    [u + tile_w, v],
                    [u + tile_w, v + tile_h],
                    [u, v + tile_h],
                ],
            )
        }
        2 => {
            // Front (+Z)
            (
                vec![
                    [x + 1.0, y, z + 1.0, default_ao],
                    [x + 1.0, y + 1.0, z + 1.0, default_ao],
                    [x, y + 1.0, z + 1.0, default_ao],
                    [x, y, z + 1.0, default_ao],
                ],
                vec![
                    [u + tile_w, v],
                    [u + tile_w, v + tile_h],
                    [u, v + tile_h],
                    [u, v],
                ],
            )
        }
        3 => {
            // Back (-Z)
            (
                vec![
                    [x, y, z, default_ao],
                    [x, y + 1.0, z, default_ao],
                    [x + 1.0, y + 1.0, z, default_ao],
                    [x + 1.0, y, z, default_ao],
                ],
                vec![
                    [u, v],
                    [u, v + tile_h],
                    [u + tile_w, v + tile_h],
                    [u + tile_w, v],
                ],
            )
        }
        4 => {
            // Right (+X)
            (
                vec![
                    [x + 1.0, y, z + 1.0, default_ao],
                    [x + 1.0, y, z, default_ao],
                    [x + 1.0, y + 1.0, z, default_ao],
                    [x + 1.0, y + 1.0, z + 1.0, default_ao],
                ],
                vec![
                    [u, v],
                    [u + tile_w, v],
                    [u + tile_w, v + tile_h],
                    [u, v + tile_h],
                ],
            )
        }
        5 => {
            // Left (-X)
            (
                vec![
                    [x, y, z, default_ao],
                    [x, y, z + 1.0, default_ao],
                    [x, y + 1.0, z + 1.0, default_ao],
                    [x, y + 1.0, z, default_ao],
                ],
                vec![
                    [u, v],
                    [u + tile_w, v],
                    [u + tile_w, v + tile_h],
                    [u, v + tile_h],
                ],
            )
        }
        _ => (vec![], vec![]),
    }
}