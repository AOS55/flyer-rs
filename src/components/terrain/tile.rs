use super::BiomeType;
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct TerrainTileComponent {
    pub biome_type: BiomeType,
    pub position: Vec2,
    pub sprite_index: usize,
}
