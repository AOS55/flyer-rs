use crate::systems::terrain::noise::NoiseLayer;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct NoiseConfig {
    pub height: HeightNoiseConfig,
    pub moisture: MoistureNoiseConfig,
    pub river: RiverNoiseConfig,
}

#[derive(Clone, Debug)]
pub struct HeightNoiseConfig {
    pub scale: f32,
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub layers: Vec<NoiseLayer>,
}

#[derive(Clone, Debug)]
pub struct MoistureNoiseConfig {
    pub scale: f32,
    pub layers: Vec<NoiseLayer>,
}

#[derive(Clone, Debug)]
pub struct RiverNoiseConfig {
    pub min_source_height: f32,
    pub meander_factor: f32,
    pub min_slope: f32,
    pub width_growth_rate: f32,
    pub depth_growth_rate: f32,
    pub erosion_strength: f32,
    pub max_length: f32,
}

impl Default for HeightNoiseConfig {
    fn default() -> Self {
        Self {
            scale: 800.0,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
            layers: vec![
                // Mountain ranges - very large scale variations
                NoiseLayer::new(2400.0, 1.0, 2)
                    .with_persistence(0.7)
                    .with_offset(Vec2::new(0.0, 0.0))
                    .with_weight(0.4),
                // Continental shapes - large, smooth variations
                NoiseLayer::new(1200.0, 1.0, 2)
                    .with_persistence(0.7)
                    .with_offset(Vec2::new(1000.0, 1000.0))
                    .with_weight(0.3),
                // Medium terrain features
                NoiseLayer::new(400.0, 0.5, 3)
                    .with_persistence(0.6)
                    .with_offset(Vec2::new(2000.0, 2000.0))
                    .with_weight(0.2),
                // Local terrain details
                NoiseLayer::new(100.0, 0.25, 4)
                    .with_persistence(0.5)
                    .with_offset(Vec2::new(3000.0, 3000.0))
                    .with_weight(0.1),
            ],
        }
    }
}

impl Default for MoistureNoiseConfig {
    fn default() -> Self {
        Self {
            scale: 250.0,
            layers: vec![
                // Large scale climate zones
                NoiseLayer::new(1000.0, 1.5, 2).with_weight(0.6),
                // Local variations
                NoiseLayer::new(600.0, 0.8, 3).with_weight(0.4),
            ],
        }
    }
}

impl Default for RiverNoiseConfig {
    fn default() -> Self {
        Self {
            min_source_height: 0.6,
            meander_factor: 0.3,
            min_slope: 0.05,
            width_growth_rate: 0.2,
            depth_growth_rate: 0.15,
            erosion_strength: 0.3,
            max_length: 150.0,
        }
    }
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            height: HeightNoiseConfig::default(),
            moisture: MoistureNoiseConfig::default(),
            river: RiverNoiseConfig::default(),
        }
    }
}
