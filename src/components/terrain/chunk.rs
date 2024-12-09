use super::{BiomeType, TerrainFeatureComponent};
use crate::resources::terrain::TerrainState;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Debug, Clone)]
#[require(Sprite)]
pub struct TerrainChunkComponent {
    pub position: IVec2,
    pub height_map: Vec<f32>,
    pub moisture_map: Vec<f32>,
    pub biome_map: Vec<BiomeType>,
    pub features: HashMap<usize, TerrainFeatureComponent>,
}

impl TerrainChunkComponent {
    pub fn new(position: IVec2, state: &TerrainState) -> Self {
        Self {
            position,
            height_map: vec![0.0; state.chunk_tile_count()],
            moisture_map: vec![0.0; state.chunk_tile_count()],
            biome_map: vec![BiomeType::Grass; state.chunk_tile_count()],
            features: HashMap::new(),
        }
    }
}
