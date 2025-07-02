use crate::chunk::{self, Chunk};
use crate::utils::tree_gen::{generate_mahogany_tree, generate_spruce_tree, generate_walnut_tree};
use crate::voxel::VoxelType;
use ferrousgl::{GlWindow, Shader};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

enum ChunkWorkerAction {
    Generate((i32, i32, i32), Option<Vec<(i32, i32, i32, VoxelType)>>), // <-- changed
    Remesh(Chunk),
    ModifyVoxel {
        chunk: Chunk,
        wx: i32,
        wy: i32,
        wz: i32,
        voxel_type: VoxelType,
    },
    ModifyVoxelsBatch {
        updates: Vec<(i32, i32, i32, VoxelType)>,
        offset: (i32, i32, i32),
        chunks: HashMap<(i32, i32, i32), Chunk>,
    },
    Exit,
}

pub struct World {
    chunks: Arc<Mutex<HashMap<(i32, i32, i32), Chunk>>>,
    pub chunk_size: usize,
    work_sender: Sender<ChunkWorkerAction>,
    chunk_receiver: Receiver<Chunk>,
    worker_handles: Vec<thread::JoinHandle<()>>,
    pub seed: u32,
    next_chunk_pos: (i32, i32, i32),
    // --- Add this field ---
    pending_voxels: Arc<Mutex<HashMap<(i32, i32, i32), Vec<(i32, i32, i32, VoxelType)>>>>,
    pending_chunks: Arc<Mutex<HashSet<(i32, i32, i32)>>>, // <-- Add this
}

impl World {
    pub fn new(chunk_size: usize, seed: u32) -> Self {
        let (work_sender, work_receiver) = channel();
        let (chunk_sender, chunk_receiver) = channel();

        let chunks = Arc::new(Mutex::new(HashMap::new()));
        let work_receiver = Arc::new(Mutex::new(work_receiver));
        let pending_voxels = Arc::new(Mutex::new(HashMap::new()));
        let pending_chunks = Arc::new(Mutex::new(HashSet::new()));

        let mut worker_handles = Vec::new();

        for _ in 0..10 {
            let work_receiver = Arc::clone(&work_receiver);
            let chunk_sender = chunk_sender.clone();
            let pending_voxels = Arc::clone(&pending_voxels);
            let pending_chunks = Arc::clone(&pending_chunks);

            let handle = thread::spawn(move || {
                loop {
                    let work = {
                        let lock = work_receiver.lock().unwrap();
                        lock.recv()
                    };
                    match work {
                        Ok(ChunkWorkerAction::Generate((cx, cy, cz), pending)) => {
                            let chunk = if let Some(pending_voxels) = pending {
                                Chunk::new(cx, cy, cz, chunk_size, seed, Some(&pending_voxels))
                            } else {
                                Chunk::new(cx, cy, cz, chunk_size, seed, None)
                            };
                            let mut chunk = chunk;
                            chunk.generate_data(cx, cy, cz);
                            chunk.prepare_mesh();
                            chunk_sender.send(chunk).unwrap();
                        }
                        Ok(ChunkWorkerAction::Remesh(mut chunk)) => {
                            chunk.prepare_mesh();
                            chunk_sender.send(chunk).unwrap();
                        }
                        Ok(ChunkWorkerAction::ModifyVoxel {
                            mut chunk,
                            wx,
                            wy,
                            wz,
                            voxel_type,
                        }) => {
                            let cs = 32 as i32;
                            let lx = wx.rem_euclid(cs) as usize;
                            let ly = wy.rem_euclid(cs) as usize;
                            let lz = wz.rem_euclid(cs) as usize;
                            chunk.set_voxel(lx, ly, lz, voxel_type);
                            chunk.prepare_mesh();
                            chunk_sender.send(chunk).unwrap();
                        }
                        Ok(ChunkWorkerAction::ModifyVoxelsBatch {
                            updates,
                            offset,
                            mut chunks,
                        }) => {
                            let chunk_size = chunk_size as i32;
                            let (offset_x, offset_y, offset_z) = offset;

                            let mut chunk_updates: HashMap<
                                (i32, i32, i32),
                                Vec<(usize, usize, usize, VoxelType)>,
                            > = HashMap::new();

                            for (x, y, z, voxel_type) in updates {
                                let wx = x + offset_x;
                                let wy = y + offset_y;
                                let wz = z + offset_z;

                                let cx = wx.div_euclid(chunk_size);
                                let cy = wy.div_euclid(chunk_size);
                                let cz = wz.div_euclid(chunk_size);
                                let lx = wx.rem_euclid(chunk_size) as usize;
                                let ly = wy.rem_euclid(chunk_size) as usize;
                                let lz = wz.rem_euclid(chunk_size) as usize;

                                chunk_updates
                                    .entry((cx, cy, cz))
                                    .or_insert_with(Vec::new)
                                    .push((lx, ly, lz, voxel_type));
                            }

                            for (chunk_coords, updates) in chunk_updates {
                                if let Some(mut chunk) = chunks.remove(&chunk_coords) {
                                    for (lx, ly, lz, voxel_type) in updates {
                                        chunk.set_voxel(lx, ly, lz, voxel_type);
                                    }
                                    chunk.prepare_mesh();
                                    chunk_sender.send(chunk).unwrap();
                                }
                            }
                        }
                        Ok(ChunkWorkerAction::Exit) | Err(_) => break,
                    }
                }
            });
            worker_handles.push(handle);
        }

        Self {
            chunks,
            chunk_size,
            work_sender,
            chunk_receiver,
            worker_handles,
            seed,
            next_chunk_pos: (0, 0, 0),
            pending_voxels,
            pending_chunks,
        }
    }

    fn drop(&mut self) {
        for _ in 0..self.worker_handles.len() {
            self.work_sender.send(ChunkWorkerAction::Exit).unwrap();
        }
        for handle in self.worker_handles.drain(..) {
            handle.join().unwrap();
        }
    }

    pub fn create_chunk(&self, cx: i32, cy: i32, cz: i32) {
        // --- Get pending voxels for this chunk, if any ---
        let pending = {
            let mut pending = self.pending_voxels.lock().unwrap();
            pending.remove(&(cx, cy, cz))
        };
        self.work_sender
            .send(ChunkWorkerAction::Generate((cx, cy, cz), pending))
            .unwrap();
    }

    pub fn request_remesh(&self, cx: i32, cy: i32, cz: i32) {
        let mut chunks = self.chunks.lock().unwrap();
        if let Some(chunk) = chunks.remove(&(cx, cy, cz)) {
            self.work_sender
                .send(ChunkWorkerAction::Remesh(chunk))
                .unwrap();
        }
    }

    pub fn remesh_now(&self, cx: i32, cy: i32, cz: i32) -> bool {
        let mut chunks = self.chunks.lock().unwrap();
        if let Some(chunk) = chunks.get_mut(&(cx, cy, cz)) {
            chunk.prepare_mesh();
            chunk.upload_to_gpu();
            true
        } else {
            false
        }
    }

    pub fn process_chunk_updates(&self, window: &GlWindow) {
        // Step 1: Collect all available chunk updates
        let mut received_chunks = Vec::new();
        while let Ok(mut chunk) = self.chunk_receiver.try_recv() {
            let key = (chunk.position.0, chunk.position.1, chunk.position.2);

            // Remove from pending_chunks
            {
                let mut pending = self.pending_chunks.lock().unwrap();
                pending.remove(&key);
            }

            // --- Save out-of-bounds voxels for other chunks ---
            let out_voxels = std::mem::take(&mut chunk.out_of_bounds_voxels);
            if !out_voxels.is_empty() {
                let cs = self.chunk_size as i32;
                let mut pending = self.pending_voxels.lock().unwrap();
                for (wx, wy, wz, vtype) in out_voxels {
                    let target_chunk = (wx.div_euclid(cs), wy.div_euclid(cs), wz.div_euclid(cs));
                    pending
                        .entry(target_chunk)
                        .or_default()
                        .push((wx, wy, wz, vtype));
                }
            }

            chunk.upload_to_gpu();
            received_chunks.push((key, chunk));
        }

        // Step 2: Insert received chunks
        {
            let mut chunks = self.chunks.lock().unwrap();
            for (key, chunk) in received_chunks {
                chunks.insert(key, chunk);
            }
        }

        // Step 2.1: Apply pending voxels to loaded chunks
        {
            let mut chunks = self.chunks.lock().unwrap();
            let mut pending = self.pending_voxels.lock().unwrap();
            let mut to_remesh = Vec::new();

            for (&key, chunk) in chunks.iter_mut() {
                if let Some(voxels) = pending.remove(&key) {
                    let cs = self.chunk_size as i32;
                    for (wx, wy, wz, vtype) in voxels {
                        let lx = wx.rem_euclid(cs) as usize;
                        let ly = wy.rem_euclid(cs) as usize;
                        let lz = wz.rem_euclid(cs) as usize;
                        chunk.set_voxel(lx, ly, lz, vtype);
                    }
                    to_remesh.push(key);
                }
            }
            drop(chunks); // Release lock before sending work

            for (cx, cy, cz) in to_remesh {
                self.request_remesh(cx, cy, cz);
            }
        }

        // Step 2.5: Rebuild chunks that need it
        {
            let mut chunks = self.chunks.lock().unwrap();
            let mut to_remesh = Vec::new();
            for (&(cx, cy, cz), chunk) in chunks.iter_mut() {
                if chunk.needs_rebuild {
                    chunk.needs_rebuild = false; // Reset the flag
                    to_remesh.push((cx, cy, cz));
                }
            }
            drop(chunks); // Release lock before sending work

            for (cx, cy, cz) in to_remesh {
                self.request_remesh(cx, cy, cz);
            }
        }

        // Decoration and detail placement removed.
        // No further steps needed.
    }

    pub fn set_voxel_main_thread(&self, wx: i32, wy: i32, wz: i32, voxel_type: VoxelType) -> bool {
        let cs = self.chunk_size as i32;
        let cx = wx.div_euclid(cs);
        let cy = wy.div_euclid(cs);
        let cz = wz.div_euclid(cs);
        let lx = wx.rem_euclid(cs) as usize;
        let ly = wy.rem_euclid(cs) as usize;
        let lz = wz.rem_euclid(cs) as usize;

        let mut chunks = self.chunks.lock().unwrap();
        let mut updated = false;

        if let Some(chunk) = chunks.get_mut(&(cx, cy, cz)) {
            chunk.set_voxel(lx, ly, lz, voxel_type);
            chunk.prepare_mesh();
            chunk.upload_to_gpu();
            updated = true;
        }

        // Check if the voxel is on any border and update neighbor meshes
        let borders = [
            (lx == 0, (-1, 0, 0)),
            (lx == cs as usize - 1, (1, 0, 0)),
            (ly == 0, (0, -1, 0)),
            (ly == cs as usize - 1, (0, 1, 0)),
            (lz == 0, (0, 0, -1)),
            (lz == cs as usize - 1, (0, 0, 1)),
        ];

        for (is_border, (dx, dy, dz)) in borders.iter().copied() {
            if is_border {
                if let Some(neighbor) = chunks.get_mut(&(cx + dx, cy + dy, cz + dz)) {
                    neighbor.prepare_mesh();
                    neighbor.upload_to_gpu();
                }
            }
        }

        updated
    }

    pub fn set_voxel_async(&self, wx: i32, wy: i32, wz: i32, voxel_type: VoxelType) -> bool {
        let cs = self.chunk_size as i32;
        let cx = wx.div_euclid(cs);
        let cy = wy.div_euclid(cs);
        let cz = wz.div_euclid(cs);

        let mut chunks = self.chunks.lock().unwrap();
        if let Some(chunk) = chunks.remove(&(cx, cy, cz)) {
            drop(chunks);
            self.work_sender
                .send(ChunkWorkerAction::ModifyVoxel {
                    chunk,
                    wx,
                    wy,
                    wz,
                    voxel_type,
                })
                .unwrap();
            true
        } else {
            false
        }
    }

    pub fn load_voxels_world_async(
        &self,
        voxels: Vec<(i32, i32, i32, VoxelType)>,
        offset: (i32, i32, i32),
    ) {
        if voxels.is_empty() {
            return;
        }

        let chunk_size = self.chunk_size as i32;
        let (offset_x, offset_y, offset_z) = offset;

        // Determine which chunks are affected
        let mut chunk_positions = std::collections::HashSet::new();
        for (x, y, z, _) in &voxels {
            let wx = x + offset_x;
            let wy = y + offset_y;
            let wz = z + offset_z;
            let cx = wx.div_euclid(chunk_size);
            let cy = wy.div_euclid(chunk_size);
            let cz = wz.div_euclid(chunk_size);
            chunk_positions.insert((cx, cy, cz));
        }

        // Collect the affected chunks from world (and remove them to move into thread)
        let mut chunk_map = HashMap::new();
        {
            let mut chunks = self.chunks.lock().unwrap();
            for &pos in &chunk_positions {
                if let Some(chunk) = chunks.remove(&pos) {
                    chunk_map.insert(pos, chunk);
                }
            }
        }

        // Send work to a worker thread
        self.work_sender
            .send(ChunkWorkerAction::ModifyVoxelsBatch {
                updates: voxels,
                offset,
                chunks: chunk_map,
            })
            .unwrap();
    }

    pub fn get_voxel(&self, wx: i32, wy: i32, wz: i32) -> VoxelType {
        let cs = self.chunk_size as i32;
        let cx = wx.div_euclid(cs);
        let cy = wy.div_euclid(cs);
        let cz = wz.div_euclid(cs);
        let lx = wx.rem_euclid(cs) as usize;
        let ly = wy.rem_euclid(cs) as usize;
        let lz = wz.rem_euclid(cs) as usize;

        let chunks = self.chunks.lock().unwrap();
        chunks
            .get(&(cx, cy, cz))
            .map(|chunk| chunk.get_voxel(lx, ly, lz))
            .unwrap_or(VoxelType::Air)
    }

    pub fn render(&self, window: &GlWindow, shader: &Shader) {
        window.set_depth_testing(ferrousgl::DepthType::LessOrEqual);
        window.set_blend_mode(ferrousgl::BlendMode::None);
        shader.set_uniform_1f("usingAlpha", 0.0);
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
        }
        let chunks = self.chunks.lock().unwrap();
        for chunk in chunks.values() {
            if chunk.is_empty() {
                continue;
            }
            chunk.render(window, shader);
        }
        shader.set_uniform_1f("usingAlpha", 1.0);
        window.set_blend_mode(ferrousgl::BlendMode::Alpha);
        for chunk in chunks.values() {
            if chunk.is_transparent_empty() {
                continue;
            }
            chunk.render_transparent(window, shader);
        }
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
    }

    pub fn update_chunks_around_player(&self, player_cx: i32, player_cy: i32, player_cz: i32) {
        // First remove distant chunks if we have too many
        self.remove_distant_chunks(player_cx, player_cy, player_cz, 4);

        // Only generate new chunks if we're under our limit
        let chunks = self.chunks.lock().unwrap();
        if chunks.len() < 400 {
            drop(chunks);
            self.generate_nearest_missing_chunk_simple(player_cx, player_cy, player_cz);
        }
    }

    pub fn remove_distant_chunks(&self, cx: i32, cy: i32, cz: i32, render_distance: i32) {
        let mut chunks = self.chunks.lock().unwrap();

        // Remove chunks outside render distance
        chunks.retain(|&(x, y, z), _| {
            (x - cx).abs() <= render_distance
                && (y - cy).abs() <= render_distance
                && (z - cz).abs() <= render_distance
        });
    }

    pub fn generate_nearest_missing_chunk_simple(&self, cx: i32, cy: i32, cz: i32) -> bool {
        let chunks = self.chunks.lock().unwrap();
        let mut pending = self.pending_chunks.lock().unwrap();

        // We'll search in expanding spherical shells within render distance
        for d in 0i32..=5 {
            // Limit to render distance
            let d_squared = d * d;
            let mut found_any = false;
            let mut closest_pos = None;
            let mut closest_dist_sq = i32::MAX;

            // Search within a cube that contains the sphere of radius d
            for dx in -d..=d {
                for dy in -d..=d {
                    for dz in -d..=d {
                        let dist_sq = dx * dx + dy * dy + dz * dz;

                        // Only consider positions within the current spherical shell
                        if dist_sq > d_squared {
                            continue;
                        }

                        let pos = (cx + dx, cy + dy, cz + dz);
                        if !chunks.contains_key(&pos) && !pending.contains(&pos) {
                            found_any = true;
                            // Track the closest position in this shell
                            if dist_sq < closest_dist_sq {
                                closest_dist_sq = dist_sq;
                                closest_pos = Some(pos);
                            }
                        }
                    }
                }
            }

            if let Some(pos) = closest_pos {
                pending.insert(pos);
                drop(chunks);
                drop(pending);
                self.create_chunk(pos.0, pos.1, pos.2);
                return true;
            }
        }

        false
    }
}
