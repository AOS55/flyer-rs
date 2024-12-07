use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone)]
pub struct TerrainFeatureComponent {
    pub feature_type: FeatureType,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: f32,
}

/// FeatureType organized into variants to help with organization
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum FeatureType {
    Tree(TreeVariant),
    Bush(BushVariant),
    Flower(FlowerVariant),
    Snow(SnowVariant),
    Rock(RockVariant),
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum TreeVariant {
    AppleTree,
    PrunedTree,
    EvergreenFir,
    WiltingFir,
    CoconutPalm,
    Palm,
    BananaTree,
    Cactus,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum BushVariant {
    GreenBushel,
    RipeBushel,
    DeadBushel,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum FlowerVariant {
    BerryBush,
    MushroomCluster,
    Reeds,
    WildFlower,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum SnowVariant {
    SnowMan,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum RockVariant {
    Log,
    RoundRock,
    CrackedRock,
    IrregularRock,
    BrownRock,
    JaggedRock,
}
