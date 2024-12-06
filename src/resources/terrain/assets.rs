use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct TerrainAssets {
    pub tile_texture: Handle<Image>,
    pub feature_texture: Handle<Image>,
    pub tile_layout: Handle<TextureAtlasLayout>,
    pub feature_layout: Handle<TextureAtlasLayout>,
}

impl TerrainAssets {
    pub fn new() -> Self {
        Self {
            tile_texture: default(),
            feature_texture: default(),
            tile_layout: default(),
            feature_layout: default(),
        }
    }
}
