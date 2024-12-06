use super::{FeatureType, FeatureVariant};
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct TerrainFeatureComponent {
    pub feature_type: FeatureType,
    pub variant: FeatureVariant,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: f32,
}
