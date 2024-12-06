use crate::components::terrain::BiomeType;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BiomeConfig {
    pub thresholds: BiomeThresholds,
    pub weights: HashMap<BiomeType, f32>,
}

#[derive(Clone, Debug)]
pub struct BiomeThresholds {
    pub water: f32,
    pub beach_width: f32,
    pub forest_moisture: f32,
}

impl Default for BiomeConfig {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert(BiomeType::Grass, 1.0);
        weights.insert(BiomeType::Forest, 0.8);
        weights.insert(BiomeType::Crops, 0.4);
        weights.insert(BiomeType::Orchard, 0.3);
        weights.insert(BiomeType::Water, 0.2);
        weights.insert(BiomeType::Sand, 0.1);

        Self {
            thresholds: BiomeThresholds {
                water: 0.45,
                beach_width: 0.025,
                forest_moisture: 0.5,
            },
            weights,
        }
    }
}
