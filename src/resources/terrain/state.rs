use bevy::prelude::*;
use std::collections::HashSet;

/// Represents the runtime and core parameters of the terrain system.
/// The `TerrainState` manages active terrain chunks, chunk loading/unloading,
/// and provides utility methods for coordinate conversions.
#[derive(Resource, Clone)]
pub struct TerrainState {
    /// Runtime state
    /// Active chunks currently loaded in the scene
    pub active_chunks: Vec<IVec2>,
    /// Chunks to load in the next frame
    pub chunks_to_load: HashSet<IVec2>,
    /// Chunks to unload in the next frame
    pub chunks_to_unload: HashSet<IVec2>,

    /// Core parameters
    /// Size of a chunk in tiles
    pub chunk_size: usize,
    /// Size of a tile in world units (m)
    pub tile_size: f32,
    /// Scale factor for converting between world and pixel units
    pub scale: f32, // px to meter scale, TODO: implement conversions

    /// Loading parameters
    /// Radius around the camera to load chunks
    pub loading_radius: i32,
    /// Maximum number of chunks to load per frame
    pub max_chunks_per_frame: usize,
}

impl TerrainState {
    /// Returns the size of a chunk in world units.
    ///
    /// # Formula:
    /// `chunk_world_size = chunk_size * tile_size`
    ///
    /// # Example:
    /// If `chunk_size = 16` and `tile_size = 2.0`, then the chunk world size is `32.0`.
    pub fn chunk_world_size(&self) -> f32 {
        self.chunk_size as f32 * self.tile_size
    }

    /// Returns the size of a single tile in world units.
    pub fn tile_world_size(&self) -> f32 {
        self.tile_size
    }

    /// Converts a world position (Vec2) to chunk coordinates (IVec2).
    ///
    /// # Arguments:
    /// * `world_pos` - The position in world units.
    ///
    /// # Returns:
    /// The chunk coordinates where the given world position lies.
    pub fn world_to_chunk(&self, world_pos: Vec2) -> IVec2 {
        let chunk_size = self.chunk_world_size();
        let chunk_pos = IVec2::new(
            (world_pos.x / chunk_size).floor() as i32,
            (world_pos.y / chunk_size).floor() as i32,
        );
        chunk_pos
    }

    /// Converts chunk coordinates (IVec2) to the world position of the chunk origin.
    ///
    /// # Arguments:
    /// * `chunk_pos` - The chunk coordinates.
    ///
    /// # Returns:
    /// The world position (bottom-left corner) of the chunk.
    pub fn chunk_to_world(&self, chunk_pos: IVec2) -> Vec2 {
        let chunk_size = self.chunk_world_size();
        let world_pos = Vec2::new(
            chunk_pos.x as f32 * chunk_size,
            chunk_pos.y as f32 * chunk_size,
        );
        world_pos
    }

    /// Converts local tile coordinates within a chunk to a position in world units.
    ///
    /// # Arguments:
    /// * `tile_x` - Tile x-coordinate within the chunk.
    /// * `tile_y` - Tile y-coordinate within the chunk.
    ///
    /// # Returns:
    /// The local position of the tile within the chunk in world units.
    pub fn tile_to_local(&self, tile_x: usize, tile_y: usize) -> Vec2 {
        Vec2::new(
            tile_x as f32 * self.tile_size,
            tile_y as f32 * self.tile_size,
        )
    }

    /// Converts a local position within a chunk to a world position.
    ///
    /// # Arguments:
    /// * `chunk_pos` - Chunk coordinates.
    /// * `local_pos` - Local position within the chunk.
    ///
    /// # Returns:
    /// The world position of the local point.
    pub fn local_to_world(&self, chunk_pos: IVec2, local_pos: Vec2) -> Vec2 {
        let world_pos = self.chunk_to_world(chunk_pos) + local_pos;
        if local_pos.x == 0.0 && local_pos.y == 0.0 {}
        world_pos
    }

    /// Returns the total number of tiles in a chunk.
    ///
    /// # Formula:
    /// `chunk_tile_count = chunk_size * chunk_size`
    pub fn chunk_tile_count(&self) -> usize {
        (self.chunk_size * self.chunk_size) as usize
    }

    /// Converts a tile index to a local position within a chunk.
    ///
    /// # Arguments:
    /// * `index` - The tile index.
    ///
    /// # Returns:
    /// The local position of the tile within the chunk.
    pub fn tile_index_to_local(&self, index: usize) -> Vec2 {
        let tile_x = index % self.chunk_size as usize;
        let tile_y = index / self.chunk_size as usize;
        self.tile_to_local(tile_x, tile_y)
    }

    /// Returns the world position of a specific tile within a chunk.
    ///
    /// # Arguments:
    /// * `chunk_pos` - The chunk coordinates.
    /// * `tile_x` - Tile x-coordinate within the chunk.
    /// * `tile_y` - Tile y-coordinate within the chunk.
    ///
    /// # Returns:
    /// The world position of the specified tile.
    pub fn get_tile_world_pos(&self, chunk_pos: IVec2, tile_x: usize, tile_y: usize) -> Vec2 {
        let chunk_world_pos = self.chunk_to_world(chunk_pos);
        let tile_offset = Vec2::new(
            tile_x as f32 * self.tile_size,
            tile_y as f32 * self.tile_size,
        );
        chunk_world_pos + tile_offset
    }

    /// Converts a tile index to its world position.
    ///
    /// # Arguments:
    /// * `chunk_pos` - The chunk coordinates.
    /// * `index` - The tile index within the chunk.
    ///
    /// # Returns:
    /// The world position of the specified tile.
    pub fn tile_index_to_world(&self, chunk_pos: IVec2, index: usize) -> Vec2 {
        let tile_x = index % self.chunk_size as usize;
        let tile_y = index / self.chunk_size as usize;
        self.get_tile_world_pos(chunk_pos, tile_x, tile_y)
    }

    /// Converts a tile index to its chunk-local position in world units.
    ///
    /// # Arguments:
    /// * `index` - The tile index within the chunk.
    ///
    /// # Returns:
    /// The position of the tile relative to the chunk origin in world units.
    pub fn tile_index_to_chunk(&self, index: usize) -> Vec2 {
        let tile_x = index % self.chunk_size as usize;
        let tile_y = index / self.chunk_size as usize;
        Vec2::new(tile_x as f32, tile_y as f32) * self.tile_size
    }

    /// Prints the debug sizes of the terrain state, including chunk size, tile size, and scale.
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
