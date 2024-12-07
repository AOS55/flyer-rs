use bevy::prelude::*;

use crate::components::terrain::*;
use crate::resources::terrain::{TerrainAssets, TerrainConfig};

pub struct TerrainRenderPlugin;

impl Plugin for TerrainRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (terrain_visual_update_system, feature_visual_update_system),
        );
    }
}

fn terrain_visual_update_system(
    chunks: Query<(&TerrainChunkComponent, &Children), Changed<TerrainChunkComponent>>,
    mut tiles: Query<(&mut Sprite, &mut Transform, &TerrainTileComponent)>,
    assets: Res<TerrainAssets>,
    config: Res<TerrainConfig>,
) {
    let tile_size = config.render.tile_size;

    for (chunk, children) in chunks.iter() {
        let chunk_size = (chunk.height_map.len() as f32).sqrt() as u32;
        let chunk_world_pos = Vec2::new(
            chunk.position.x as f32 * chunk_size as f32 * tile_size,
            chunk.position.y as f32 * chunk_size as f32 * tile_size,
        );

        for (index, &child) in children.iter().enumerate() {
            if let Ok((mut sprite, mut transform, _)) = tiles.get_mut(child) {
                let x = index as u32 % chunk_size;
                let y = index as u32 / chunk_size;
                let idx = y * chunk_size + x;

                // Update sprite based on biome
                let biome = chunk.biome_map[idx as usize];
                if let Some(sprite_index) = get_sprite_index(biome, &assets) {
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: assets.tile_layout.clone(),
                        index: sprite_index,
                    });
                }

                // Position
                let world_x = chunk_world_pos.x + x as f32 * tile_size;
                let world_y = chunk_world_pos.y + y as f32 * tile_size;
                transform.translation = Vec3::new(world_x, world_y, get_biome_z_layer(biome));

                // Height-based scaling
                let height = chunk.height_map[idx as usize];
                let scale = 1.0 + (height * 0.2);
                transform.scale = Vec3::splat(scale);
            }
        }
    }
}

fn feature_visual_update_system(
    mut features: Query<(&TerrainFeatureComponent, &mut Sprite, &mut Transform)>,
    assets: Res<TerrainAssets>,
    config: Res<TerrainConfig>,
) {
    for (feature, mut sprite, mut transform) in features.iter_mut() {
        // Update sprite
        if let Some(sprite_index) = get_feature_sprite_index(feature.feature_type, &assets) {
            sprite.texture_atlas = Some(TextureAtlas {
                layout: assets.feature_layout.clone(),
                index: sprite_index,
            });
        }

        // Position and scale
        transform.translation.z = get_feature_z_layer(feature.feature_type);
        transform.scale = Vec3::splat(feature.scale);
        transform.rotation = Quat::from_rotation_z(feature.rotation);
    }
}

fn get_sprite_index(biome: BiomeType, assets: &TerrainAssets) -> Option<usize> {
    assets.tile_mappings.get(&biome).copied()
}

fn get_feature_sprite_index(feature_type: FeatureType, assets: &TerrainAssets) -> Option<usize> {
    assets.feature_mappings.get(&feature_type).copied()
}

fn get_biome_z_layer(biome: BiomeType) -> f32 {
    match biome {
        BiomeType::Water => 0.0,
        BiomeType::Beach => 1.0,
        BiomeType::Grass | BiomeType::Crops => 2.0,
        BiomeType::Forest | BiomeType::Orchard => 3.0,
        BiomeType::Desert => 0.0,
        BiomeType::Mountain => 0.0,
        BiomeType::Snow => 5.0,
    }
}

fn get_feature_z_layer(feature_type: FeatureType) -> f32 {
    match feature_type {
        FeatureType::Bush(_) => 3.0,
        FeatureType::Flower(_) => 3.1,
        FeatureType::Tree(_) => 4.0,
        FeatureType::Snow(_) => 2.0,
        FeatureType::Rock(_) => 2.5,
    }
}
