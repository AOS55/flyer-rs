use bevy::prelude::*;

use crate::components::terrain::*;
use crate::resources::terrain::{TerrainAssets, TerrainConfig, TerrainState};

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
    state: Res<TerrainState>,
    assets: Res<TerrainAssets>,
) {
    for (chunk, children) in chunks.iter() {
        // Use state to compute the chunk's world position
        let chunk_world_pos = state.chunk_to_world(chunk.position);

        for (index, &child) in children.iter().enumerate() {
            if let Ok((mut sprite, mut transform, tile)) = tiles.get_mut(child) {
                // Calculate x,y grid position within chunk
                let x = index % state.chunk_size;
                let y = index / state.chunk_size;

                // Update sprite based on biome
                if let Some(&sprite_index) = assets.tile_mappings.get(&tile.biome_type) {
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: assets.tile_layout.clone(),
                        index: sprite_index,
                    });
                }

                // Position each tile by its grid position times tile size
                let world_x = chunk_world_pos.x + x as f32 * state.tile_size;
                let world_y = chunk_world_pos.y + y as f32 * state.tile_size;
                transform.translation =
                    Vec3::new(world_x, world_y, get_biome_z_layer(tile.biome_type));

                // Keep scale at 1 to prevent gaps
                transform.scale = Vec3::splat(1.0);
            }
        }
    }
}

fn feature_visual_update_system(
    mut features: Query<(&TerrainFeatureComponent, &mut Sprite, &mut Transform)>,
    assets: Res<TerrainAssets>,
    _config: Res<TerrainConfig>,
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
