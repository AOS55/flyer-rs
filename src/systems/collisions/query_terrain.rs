use crate::{
    components::terrain::{BiomeType, TerrainChunkComponent, TerrainFeatureComponent},
    resources::TerrainState,
};
use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct TerrainInfo {
    pub height: f32,
    pub moisture: f32,
    pub biome: BiomeType,
    pub features: Vec<TerrainFeatureComponent>,
}

/// System to query terrain information at a specific world position
pub fn get_terrain_at_position(
    world_pos: Vec2,
    chunks: &Query<(&TerrainChunkComponent, &Transform)>,
    state: &TerrainState,
) -> Option<TerrainInfo> {
    // Convert world position to chunk coordinates
    let chunk_pos = state.world_to_chunk(world_pos);

    // Find the chunk containing this position
    let chunk = chunks
        .iter()
        .find(|(chunk, _)| chunk.position == chunk_pos)?;
    let (chunk_component, _) = chunk;

    // Calculate local position within chunk
    let chunk_world_pos = state.chunk_to_world(chunk_pos);
    let local_pos = world_pos - chunk_world_pos;

    // Convert local position to tile coordinates
    let tile_x = (local_pos.x / state.tile_size).floor() as usize;
    let tile_y = (local_pos.y / state.tile_size).floor() as usize;

    // Get tile index
    let tile_idx = tile_y * state.chunk_size + tile_x;

    // Ensure we're within bounds
    if tile_idx >= chunk_component.height_map.len() {
        return None;
    }

    // Collect any features at this position
    let features = chunk_component
        .features
        .iter()
        .filter(|(&idx, _)| idx == tile_idx)
        .map(|(_, feature)| feature.clone())
        .collect();

    Some(TerrainInfo {
        height: chunk_component.height_map[tile_idx],
        moisture: chunk_component.moisture_map[tile_idx],
        biome: chunk_component.biome_map[tile_idx],
        features,
    })
}
