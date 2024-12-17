use crate::systems::terrain::noise::NoiseLayer;
use bevy::prelude::*;

/// Configuration for different noise layers used to generate procedural terrain.
/// The noise configuration includes separate settings for height, moisture, and rivers,
/// allowing fine control over the appearance and features of the terrain.
#[derive(Clone, Debug)]
pub struct NoiseConfig {
    /// Configuration for height noise, which controls the terrain elevation.
    pub height: HeightNoiseConfig,
    /// Configuration for moisture noise, which determines the distribution of biomes or climate.
    pub moisture: MoistureNoiseConfig,
    /// Configuration for river noise, which defines river shapes, flow, and related features.
    pub river: RiverNoiseConfig,
}

/// Configuration for generating height-based terrain noise.
/// Controls the terrain elevation through multiple noise layers with varying scales and details.
#[derive(Clone, Debug)]
pub struct HeightNoiseConfig {
    /// The overall scale of the height noise. Larger values produce smoother, broader features.
    pub scale: f32,
    /// The number of noise octaves used. Each octave adds finer detail at smaller scales.
    pub octaves: u32,
    /// The persistence factor for successive octaves. Lower values reduce the contribution of higher octaves.
    pub persistence: f32,
    /// The lacunarity factor, which determines the frequency of each successive octave.
    /// Larger values make higher octaves have greater frequency.
    pub lacunarity: f32,
    /// A list of noise layers defining how different scales of noise combine to form the height map.
    pub layers: Vec<NoiseLayer>,
}

/// Configuration for generating moisture-based noise.
/// Moisture noise is used to simulate climate zones and biome distribution.
#[derive(Clone, Debug)]
pub struct MoistureNoiseConfig {
    /// The overall scale of the moisture noise. Larger values result in broader climate zones.
    pub scale: f32,
    /// A list of noise layers that combine to create moisture variations.
    pub layers: Vec<NoiseLayer>,
}

/// Configuration for generating rivers within the terrain.
/// This controls how rivers are shaped, their flow patterns, and erosion effects.
#[derive(Clone, Debug)]
pub struct RiverNoiseConfig {
    /// The minimum source height for a river to originate.
    /// Higher values prevent rivers from starting in low-lying areas.
    pub min_source_height: f32,
    /// The meander factor, which determines how winding or straight rivers are.
    pub meander_factor: f32,
    /// The minimum slope required for river flow.
    /// Prevents rivers from forming on completely flat terrain.
    pub min_slope: f32,
    /// The rate at which the river width increases as it flows downstream.
    pub width_growth_rate: f32,
    /// The rate at which the river depth increases as it flows downstream.
    pub depth_growth_rate: f32,
    /// The strength of erosion caused by the river, affecting the surrounding terrain.
    pub erosion_strength: f32,
    /// The maximum allowable length of a river.
    /// Limits how far a river can flow across the terrain.
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
