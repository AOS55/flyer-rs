use serde::{Deserialize, Serialize};

mod chunk;
mod feature;
mod tile;

pub use chunk::TerrainChunkComponent;
pub use feature::TerrainFeatureComponent;
pub use tile::TerrainTileComponent;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum BiomeType {
    Grass,
    Forest,
    Crops,
    Orchard,
    Water,
    Sand,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum FeatureType {
    Tree(TreeVariant),
    Bush(BushVariant),
    Flower(FlowerVariant),
    Rock,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum TreeVariant {
    EvergreenFir,
    WiltingFir,
    AppleTree,
    PrunedTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum BushVariant {
    GreenBushel,
    RipeBushel,
    DeadBushel,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum FlowerVariant {
    Single,
    Double,
    Quad,
    Cluster,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum FeatureVariant {
    Tree(TreeVariant),
    Bush(BushVariant),
    Flower(FlowerVariant),
    Rock,
}
