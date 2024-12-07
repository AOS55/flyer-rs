use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Clone)]
pub struct TerrainState {
    // Runtime state
    pub active_chunks: Vec<IVec2>,
    pub chunks_to_load: HashSet<IVec2>,
    pub chunks_to_unload: HashSet<IVec2>,

    // Core parameters
    pub chunk_size: u32,
    pub scale: f32,
    pub seed: u64,

    // Loading parameters
    pub loading_radius: i32,
    pub max_chunks_per_frame: usize,
}
