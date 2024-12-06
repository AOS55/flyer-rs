mod biome;
mod feature;
mod noise;

use bevy::prelude::*;
pub use biome::BiomeConfig;
pub use feature::FeatureConfig;
pub use noise::{NoiseConfig, RiverNoiseConfig};

#[derive(Resource, Clone, Debug)]
pub struct TerrainConfig {
    pub noise: NoiseConfig,
    pub biome: BiomeConfig,
    pub feature: FeatureConfig,
    pub render: RenderConfig,
}

#[derive(Clone, Debug)]
pub struct RenderConfig {
    pub tile_size: f32,
    pub feature_layer_offset: f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            noise: NoiseConfig::default(),
            biome: BiomeConfig::default(),
            feature: FeatureConfig::default(),
            render: RenderConfig {
                tile_size: 16.0,
                feature_layer_offset: 10.0,
            },
        }
    }
}
