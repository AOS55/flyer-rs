use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::terrain::{BiomeType, FeatureType};

#[derive(Resource, Clone)]
pub struct TerrainAssets {
    pub tile_texture: Handle<Image>,
    pub feature_texture: Handle<Image>,
    pub tile_layout: Handle<TextureAtlasLayout>,
    pub feature_layout: Handle<TextureAtlasLayout>,
    pub tile_mappings: HashMap<BiomeType, usize>,
    pub feature_mappings: HashMap<FeatureType, usize>,
}

impl TerrainAssets {
    pub fn new() -> Self {
        Self {
            tile_texture: default(),
            feature_texture: default(),
            tile_layout: default(),
            feature_layout: default(),
            tile_mappings: HashMap::new(),
            feature_mappings: HashMap::new(),
        }
    }
}
