use super::BiomeType;
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct TerrainChunkComponent {
    pub position: IVec2,
    pub height_map: Vec<f32>,
    pub moisture_map: Vec<f32>,
    pub biome_map: Vec<BiomeType>,
}

impl TerrainChunkComponent {
    pub fn new(position: IVec2, chunk_size: u32) -> Self {
        let size = (chunk_size * chunk_size) as usize;
        Self {
            position,
            height_map: vec![0.0; size],
            moisture_map: vec![0.0; size],
            biome_map: vec![BiomeType::Grass; size],
        }
    }

    pub fn world_position(&self, chunk_size: u32, scale: f32) -> Vec2 {
        Vec2::new(
            self.position.x as f32 * chunk_size as f32 * scale,
            self.position.y as f32 * chunk_size as f32 * scale,
        )
    }
}
