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

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            height: HeightNoiseConfig {
                scale: 100.0,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
                layers: vec![
                    NoiseLayer::new(50.0, 0.5, 2)
                        .with_offset(Vec2::new(1000.0, 1000.0))
                        .with_weight(0.5),
                    NoiseLayer::new(25.0, 0.25, 1)
                        .with_offset(Vec2::new(2000.0, 2000.0))
                        .with_weight(0.25),
                ],
            },
            moisture: MoistureNoiseConfig {
                scale: 150.0,
                layers: vec![NoiseLayer::new(75.0, 0.5, 2).with_weight(0.3)],
            },
            river: RiverNoiseConfig {
                min_source_height: 0.7,
                meander_factor: 0.3,
                min_slope: 0.01,
                width_growth_rate: 0.1,
                depth_growth_rate: 0.05,
                erosion_strength: 0.2,
                max_length: 100.0,
            },
        }
    }
}
