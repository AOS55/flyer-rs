use crate::components::terrain::{BiomeType, BushVariant, FeatureType, FlowerVariant, TreeVariant};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct FeatureConfig {
    pub densities: HashMap<FeatureType, f32>,
    pub biome_multipliers: HashMap<(FeatureType, BiomeType), f32>,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        let mut densities = HashMap::new();
        let mut biome_multipliers = HashMap::new();

        // Base densities
        densities.insert(FeatureType::Tree(TreeVariant::EvergreenFir), 0.6);
        densities.insert(FeatureType::Tree(TreeVariant::AppleTree), 0.1);
        densities.insert(FeatureType::Bush(BushVariant::GreenBushel), 0.2);
        densities.insert(FeatureType::Flower(FlowerVariant::Single), 0.1);

        // Biome-specific multipliers
        biome_multipliers.insert(
            (
                FeatureType::Tree(TreeVariant::EvergreenFir),
                BiomeType::Forest,
            ),
            1.5,
        );
        biome_multipliers.insert(
            (
                FeatureType::Tree(TreeVariant::AppleTree),
                BiomeType::Orchard,
            ),
            1.2,
        );
        biome_multipliers.insert(
            (
                FeatureType::Bush(BushVariant::GreenBushel),
                BiomeType::Crops,
            ),
            1.3,
        );
        biome_multipliers.insert(
            (FeatureType::Flower(FlowerVariant::Single), BiomeType::Grass),
            1.1,
        );

        Self {
            densities,
            biome_multipliers,
        }
    }
}
