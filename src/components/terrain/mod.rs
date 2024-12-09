use serde::{Deserialize, Serialize};

mod chunk;
mod feature;
mod tile;

pub use chunk::TerrainChunkComponent;
pub use feature::{
    BushVariant, FeatureType, FlowerVariant, RockVariant, SnowVariant, TerrainFeatureComponent,
    TreeVariant,
};
pub use tile::TerrainTileComponent;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum BiomeType {
    Grass,
    Forest,
    Crops,
    Orchard,
    Water,
    Beach,
    Desert,
    Mountain,
    Snow,
}

impl Default for BiomeType {
    fn default() -> Self {
        BiomeType::Grass
    }
}
