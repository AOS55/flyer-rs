use bevy::prelude::*;
use flyer::{components::terrain::*, resources::terrain::*, systems::terrain::noise::NoiseLayer};
use std::sync::LazyLock;

pub mod fixtures {
    use super::*;

    pub static TEST_TERRAIN_CONFIG: LazyLock<TerrainConfig> = LazyLock::new(|| TerrainConfig {
        seed: 42,
        noise: NoiseConfig {
            height: HeightNoiseConfig {
                scale: 800.0,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
                layers: Vec::new(),
            },
            moisture: MoistureNoiseConfig {
                scale: 250.0,
                layers: Vec::new(),
            },
            river: RiverNoiseConfig {
                min_source_height: 0.6,
                meander_factor: 0.3,
                min_slope: 0.05,
                width_growth_rate: 0.2,
                depth_growth_rate: 0.15,
                erosion_strength: 0.3,
                max_length: 150.0,
            },
        },
        biome: BiomeConfig {
            thresholds: BiomeThresholds {
                water: 0.48,
                mountain_start: 0.75,
                mountain_width: 0.1,
                beach_width: 0.025,
                forest_moisture: 0.95,
                desert_moisture: 0.2,
                field_sizes: [96.0, 128.0, 256.0, 512.0],
            },
        },
        feature: FeatureConfig::default(),
        render: RenderConfig {
            feature_layer_offset: 10.0,
        },
    });
}

/// Creates a basic terrain chunk for testing
pub fn create_test_chunk(position: IVec2) -> TerrainChunkComponent {
    let mut chunk = TerrainChunkComponent {
        position,
        height_map: vec![0.5; 256],
        moisture_map: vec![0.5; 256],
        biome_map: vec![BiomeType::Grass; 256],
        features: Default::default(),
    };

    // Add some basic terrain features
    chunk.features.insert(
        0,
        TerrainFeatureComponent {
            feature_type: FeatureType::Tree(TreeVariant::EvergreenFir),
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: 1.0,
        },
    );

    chunk
}

/// Creates noise configurations for different terrain types
pub mod noise_configs {
    use super::*;

    pub fn mountain_noise() -> NoiseLayer {
        NoiseLayer::new(2400.0, 1.0, 2)
            .with_persistence(0.7)
            .with_offset(Vec2::new(0.0, 0.0))
            .with_weight(0.4)
    }

    pub fn plains_noise() -> NoiseLayer {
        NoiseLayer::new(1200.0, 0.5, 2)
            .with_persistence(0.5)
            .with_offset(Vec2::new(1000.0, 1000.0))
            .with_weight(0.3)
    }

    pub fn detail_noise() -> NoiseLayer {
        NoiseLayer::new(100.0, 0.25, 4)
            .with_persistence(0.5)
            .with_offset(Vec2::new(3000.0, 3000.0))
            .with_weight(0.1)
    }
}

/// Creates biome configurations for different terrain types
pub mod biome_configs {
    use super::*;

    pub fn mountainous() -> BiomeConfig {
        BiomeConfig {
            thresholds: BiomeThresholds {
                water: 0.35,
                mountain_start: 0.6,
                mountain_width: 0.2,
                beach_width: 0.02,
                forest_moisture: 0.8,
                desert_moisture: 0.2,
                field_sizes: [96.0, 128.0, 256.0, 512.0],
            },
        }
    }

    pub fn coastal() -> BiomeConfig {
        BiomeConfig {
            thresholds: BiomeThresholds {
                water: 0.55,
                mountain_start: 0.8,
                mountain_width: 0.1,
                beach_width: 0.04,
                forest_moisture: 0.9,
                desert_moisture: 0.3,
                field_sizes: [96.0, 128.0, 256.0, 512.0],
            },
        }
    }
}

pub mod utils {
    use super::*;
    pub fn create_tree(position: Vec2) -> TerrainFeatureComponent {
        TerrainFeatureComponent {
            feature_type: FeatureType::Tree(TreeVariant::EvergreenFir),
            position,
            rotation: 0.0,
            scale: 1.0,
        }
    }

    pub fn create_rock(position: Vec2) -> TerrainFeatureComponent {
        TerrainFeatureComponent {
            feature_type: FeatureType::Rock(RockVariant::BrownRock),
            position,
            rotation: 0.0,
            scale: 1.0,
        }
    }

    pub fn create_bush(position: Vec2) -> TerrainFeatureComponent {
        TerrainFeatureComponent {
            feature_type: FeatureType::Bush(BushVariant::GreenBushel),
            position,
            rotation: 0.0,
            scale: 1.0,
        }
    }
}
