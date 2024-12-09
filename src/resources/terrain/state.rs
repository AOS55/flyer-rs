use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Clone)]
pub struct TerrainState {
    // Runtime state
    pub active_chunks: Vec<IVec2>,
    pub chunks_to_load: HashSet<IVec2>,
    pub chunks_to_unload: HashSet<IVec2>,

    // Core parameters
    pub chunk_size: usize,
    pub tile_size: f32,
    pub scale: f32,
    pub seed: u64,

    // Loading parameters
    pub loading_radius: i32,
    pub max_chunks_per_frame: usize,
}

impl TerrainState {
    // Get the size of a chunk in world units
    pub fn chunk_world_size(&self) -> f32 {
        self.chunk_size as f32 * self.tile_size * self.scale
    }

    // Helper method to get the size of a tile in world units
    pub fn tile_world_size(&self) -> f32 {
        self.tile_size * self.scale
    }

    // Convert world position to chunk coordinates
    pub fn world_to_chunk(&self, world_pos: Vec2) -> IVec2 {
        let chunk_size = self.chunk_world_size();
        let chunk_pos = IVec2::new(
            (world_pos.x / chunk_size).floor() as i32,
            (world_pos.y / chunk_size).floor() as i32,
        );
        chunk_pos
    }

    // Convert chunk coordinates to world position (of chunk origin)
    pub fn chunk_to_world(&self, chunk_pos: IVec2) -> Vec2 {
        let chunk_size = self.chunk_world_size();
        let world_pos = Vec2::new(
            chunk_pos.x as f32 * chunk_size,
            chunk_pos.y as f32 * chunk_size,
        );
        world_pos
    }

    // Convert local tile coordinates to world position within a chunk
    pub fn tile_to_local(&self, tile_x: usize, tile_y: usize) -> Vec2 {
        Vec2::new(
            tile_x as f32 * self.tile_size,
            tile_y as f32 * self.tile_size,
        )
    }

    // Convert local position to world position
    pub fn local_to_world(&self, chunk_pos: IVec2, local_pos: Vec2) -> Vec2 {
        let world_pos = self.chunk_to_world(chunk_pos) + local_pos;
        if local_pos.x == 0.0 && local_pos.y == 0.0 {}
        world_pos
    }

    // Get total tile count for a chunk
    pub fn chunk_tile_count(&self) -> usize {
        (self.chunk_size * self.chunk_size) as usize
    }

    // Convert tile index to local position within chunk
    pub fn tile_index_to_local(&self, index: usize) -> Vec2 {
        let tile_x = index % self.chunk_size as usize;
        let tile_y = index / self.chunk_size as usize;
        self.tile_to_local(tile_x, tile_y)
    }

    // Get world position for a specific tile in a chunk
    pub fn get_tile_world_pos(&self, chunk_pos: IVec2, tile_x: usize, tile_y: usize) -> Vec2 {
        let chunk_world_pos = self.chunk_to_world(chunk_pos);
        let tile_offset = Vec2::new(
            tile_x as f32 * self.tile_size,
            tile_y as f32 * self.tile_size,
        );
        chunk_world_pos + tile_offset
    }

    // Convert tile index to world position
    pub fn tile_index_to_world(&self, chunk_pos: IVec2, index: usize) -> Vec2 {
        let tile_x = index % self.chunk_size as usize;
        let tile_y = index / self.chunk_size as usize;
        self.get_tile_world_pos(chunk_pos, tile_x, tile_y)
    }

    pub fn tile_index_to_chunk(&self, index: usize) -> Vec2 {
        let tile_x = index % self.chunk_size as usize;
        let tile_y = index / self.chunk_size as usize;
        Vec2::new(tile_x as f32, tile_y as f32) * self.tile_size
    }

    // Debug method to verify calculations
    pub fn debug_print_sizes(&self) {
        info!(
            "Terrain sizes - Chunk: {}, Tile: {}, Scale: {}, Total chunk size: {}",
            self.chunk_size,
            self.tile_size,
            self.scale,
            self.chunk_world_size()
        );
    }
}
