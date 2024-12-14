mod biome;
mod feature;
mod noise;

use bevy::prelude::*;
pub use biome::{BiomeConfig, BiomeThresholds};
pub use feature::{BiomeFeatureConfig, FeatureConfig};
pub use noise::{HeightNoiseConfig, MoistureNoiseConfig, NoiseConfig, RiverNoiseConfig};

#[derive(Resource, Clone, Debug)]
pub struct TerrainConfig {
    pub seed: u64,
    pub noise: NoiseConfig,
    pub biome: BiomeConfig,
    pub feature: FeatureConfig,
    pub render: RenderConfig,
}

#[derive(Clone, Debug)]
pub struct RenderConfig {
    pub feature_layer_offset: f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            noise: NoiseConfig::default(),
            biome: BiomeConfig::default(),
            feature: FeatureConfig::default(),
            render: RenderConfig {
                feature_layer_offset: 10.0,
            },
        }
    }
}
