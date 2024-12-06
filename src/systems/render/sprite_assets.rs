use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct SpriteAssets {
    pub textures: HashMap<String, Handle<Image>>,
    pub atlas_layouts: HashMap<String, Handle<TextureAtlasLayout>>,
    pub atlas_sources: HashMap<String, TextureAtlasSources>,
}

pub fn load_sprite_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut sprite_assets: ResMut<SpriteAssets>,
) {
    // Load textures
    sprite_assets.textures.insert(
        "player".to_string(),
        asset_server.load("sprites/player.png"),
    );

    // Create atlas layouts and sources
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 8, 4, None, None);
    let layout_handle = texture_atlas_layouts.add(layout.clone());

    // Store both layout and sources
    sprite_assets
        .atlas_layouts
        .insert("animated_sprite".to_string(), layout_handle);
}
