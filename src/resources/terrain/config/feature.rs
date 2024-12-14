use crate::components::terrain::{
    BushVariant, FeatureType, FlowerVariant, RockVariant, SnowVariant, TreeVariant,
};
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct FeatureConfig {
    pub grass: GrassFeatureConfig,
    pub forest: ForestFeatureConfig,
    pub crops: CropsFeatureConfig,
    pub orchard: OrchardFeatureConfig,
    pub water: WaterFeatureConfig,
    pub beach: BeachFeatureConfig,
    pub desert: DesertFeatureConfig,
    pub mountain: MountainFeatureConfig,
    pub snow: SnowFeatureConfig,
    pub rng: Option<StdRng>,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            grass: GrassFeatureConfig::default(),
            forest: ForestFeatureConfig::default(),
            crops: CropsFeatureConfig::default(),
            orchard: OrchardFeatureConfig::default(),
            water: WaterFeatureConfig::default(),
            beach: BeachFeatureConfig::default(),
            desert: DesertFeatureConfig::default(),
            mountain: MountainFeatureConfig::default(),
            snow: SnowFeatureConfig::default(),
            rng: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GrassFeatureConfig {
    pub density: f32, // probability of a feature being placed in a tile
    pub feature_probs: Vec<(FeatureType, f32)>, // selection of each feature type
}

#[derive(Clone, Debug)]
pub struct ForestFeatureConfig {
    pub density: f32,
    pub feature_probs: Vec<(FeatureType, f32)>,
}

#[derive(Clone, Debug)]
pub struct CropsFeatureConfig {
    pub density: f32,
    pub feature_probs: Vec<(FeatureType, f32)>,
}

#[derive(Clone, Debug)]
pub struct OrchardFeatureConfig {
    pub density: f32,
    pub feature_probs: Vec<(FeatureType, f32)>,
}

#[derive(Clone, Debug)]
pub struct WaterFeatureConfig {
    pub density: f32,
    pub feature_probs: Vec<(FeatureType, f32)>,
}

#[derive(Clone, Debug)]
pub struct BeachFeatureConfig {
    pub density: f32,
    pub feature_probs: Vec<(FeatureType, f32)>,
}

#[derive(Clone, Debug)]
pub struct DesertFeatureConfig {
    pub density: f32,
    pub feature_probs: Vec<(FeatureType, f32)>,
}

#[derive(Clone, Debug)]
pub struct MountainFeatureConfig {
    pub density: f32,
    pub feature_probs: Vec<(FeatureType, f32)>,
}

#[derive(Clone, Debug)]
pub struct SnowFeatureConfig {
    pub density: f32,
    pub feature_probs: Vec<(FeatureType, f32)>,
}

pub trait BiomeFeatureConfig {
    fn density(&self) -> f32;
    fn feature_probs(&self) -> &[(FeatureType, f32)];

    fn select_feature(&self, rng: &mut StdRng) -> Option<FeatureType> {
        let weights: Vec<f32> = self.feature_probs().iter().map(|(_, prob)| *prob).collect();
        let dist = WeightedIndex::new(&weights).ok()?;
        let index = dist.sample(rng);
        Some(self.feature_probs()[index].0.clone())
    }
}

macro_rules! impl_biome_feature_config {
    ($($t:ty),*) => {
        $(
            impl BiomeFeatureConfig for $t {
                fn density(&self) -> f32 {
                    self.density
                }

                fn feature_probs(&self) -> &[(FeatureType, f32)] {
                    &self.feature_probs
                }
            }
        )*
    };
}

impl_biome_feature_config!(
    GrassFeatureConfig,
    ForestFeatureConfig,
    CropsFeatureConfig,
    OrchardFeatureConfig,
    WaterFeatureConfig,
    BeachFeatureConfig,
    DesertFeatureConfig,
    MountainFeatureConfig,
    SnowFeatureConfig
);

impl Default for GrassFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.01,
            feature_probs: vec![
                (FeatureType::Flower(FlowerVariant::BerryBush), 0.4),
                (FeatureType::Flower(FlowerVariant::WildFlower), 0.2),
                (FeatureType::Rock(RockVariant::JaggedRock), 0.1),
            ],
        }
    }
}

impl Default for ForestFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.95,
            feature_probs: vec![
                (FeatureType::Tree(TreeVariant::EvergreenFir), 0.8),
                (FeatureType::Tree(TreeVariant::WiltingFir), 0.2),
            ],
        }
    }
}

impl Default for CropsFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.8,
            feature_probs: vec![
                (FeatureType::Bush(BushVariant::GreenBushel), 0.5),
                (FeatureType::Bush(BushVariant::RipeBushel), 0.3),
                (FeatureType::Bush(BushVariant::DeadBushel), 0.1),
            ],
        }
    }
}

impl Default for OrchardFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.8,
            feature_probs: vec![
                (FeatureType::Tree(TreeVariant::AppleTree), 0.5),
                (FeatureType::Tree(TreeVariant::PrunedTree), 0.3),
            ],
        }
    }
}

impl Default for WaterFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.00,
            feature_probs: vec![],
        }
    }
}

impl Default for BeachFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.2,
            feature_probs: vec![
                (FeatureType::Rock(RockVariant::BrownRock), 0.05),
                (FeatureType::Tree(TreeVariant::Palm), 0.6),
                (FeatureType::Tree(TreeVariant::BananaTree), 0.1),
                (FeatureType::Tree(TreeVariant::CoconutPalm), 0.3),
            ],
        }
    }
}

impl Default for DesertFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.1,
            feature_probs: vec![
                (FeatureType::Tree(TreeVariant::Cactus), 0.1),
                (FeatureType::Rock(RockVariant::IrregularRock), 0.05),
                (FeatureType::Rock(RockVariant::JaggedRock), 0.05),
            ],
        }
    }
}

impl Default for MountainFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.1,
            feature_probs: vec![
                (FeatureType::Rock(RockVariant::CrackedRock), 0.1),
                (FeatureType::Rock(RockVariant::BrownRock), 0.1),
                (FeatureType::Rock(RockVariant::IrregularRock), 0.1),
            ],
        }
    }
}

impl Default for SnowFeatureConfig {
    fn default() -> Self {
        Self {
            density: 0.02,
            feature_probs: vec![(FeatureType::Snow(SnowVariant::SnowMan), 1.0)],
        }
    }
}

impl FeatureConfig {
    pub fn with_seed(mut self, master_seed: u64) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        master_seed.hash(&mut hasher);
        "feature".hash(&mut hasher);
        let feature_seed = hasher.finish();

        self.rng = Some(StdRng::seed_from_u64(feature_seed));
        self
    }
}
