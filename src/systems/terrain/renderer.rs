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

/// System to update terrain visuals based on chunk data
// pub fn terrain_visual_update_system(
//     mut chunks: Query<(Entity, &TerrainChunkComponent, &Children), Changed<TerrainChunkComponent>>,
//     mut tiles: Query<(&mut Sprite, &mut Transform, &TerrainTileComponent)>,
//     terrain_assets: Res<TerrainAssets>,
//     render_config: Res<TerrainConfig>,
// ) {
//     for (_chunk_entity, chunk, children) in chunks.iter_mut() {
//         update_chunk_visuals(
//             chunk,
//             children,
//             &mut tiles,
//             &terrain_assets,
//             &render_config.render,
//         );

//         // Logging for debugging
//         info!(
//             "Updating visuals for chunk at position: {:?}",
//             chunk.position
//         );
//     }
// }

// Helper Functions

// fn update_chunk_visuals(
//     chunk: &TerrainChunkComponent,
//     children: &Children,
//     tiles: &mut Query<(&mut Sprite, &mut Transform, &TerrainTileComponent)>,
//     terrain_assets: &TerrainAssets,
//     render_config: &RenderConfig,
// ) {
//     let chunk_size = (chunk.biome_map.len() as f32).sqrt() as usize;
//     let tile_size = render_config.tile_size;

//     for (child_index, &child_entity) in children.iter().enumerate() {
//         if let Ok((mut sprite, mut transform, _tile)) = tiles.get_mut(child_entity) {
//             let world_pos = chunk.world_position(chunk_size as u32, render_config.tile_size);
//             let x = world_pos.x + (child_index % chunk_size) as f32 * tile_size;
//             let y = world_pos.y + (child_index / chunk_size) as f32 * tile_size;

//             // Update position
//             transform.translation =
//                 Vec3::new(x, y, get_biome_z_layer(chunk.biome_map[child_index]));

//             // Update sprite based on biome
//             if let Some(&sprite_index) = terrain_assets
//                 .tile_mappings
//                 .get(&chunk.biome_map[child_index])
//             {
//                 sprite.texture_atlas = Some(TextureAtlas {
//                     layout: terrain_assets.tile_layout.clone(),
//                     index: sprite_index,
//                 });
//             }

//             // Apply height-based scaling
//             let height_scale = 1.0 + (chunk.height_map[child_index] * 0.2);
//             transform.scale = Vec3::splat(height_scale);

//             // Logging for debugging
//             info!("Updating tile at position: {:?}", transform.translation);
//         }
//     }
// }

fn get_sprite_index(biome: BiomeType, assets: &TerrainAssets) -> Option<usize> {
    assets.tile_mappings.get(&biome).copied()
}

fn get_feature_sprite_index(feature_type: FeatureType, assets: &TerrainAssets) -> Option<usize> {
    assets.feature_mappings.get(&feature_type).copied()
}

fn get_biome_z_layer(biome: BiomeType) -> f32 {
    match biome {
        BiomeType::Water => 0.0,
        BiomeType::Sand => 1.0,
        BiomeType::Grass | BiomeType::Crops => 2.0,
        BiomeType::Forest | BiomeType::Orchard => 3.0,
    }
}

fn get_feature_z_layer(feature_type: FeatureType) -> f32 {
    match feature_type {
        FeatureType::Bush(_) => 3.0,
        FeatureType::Flower(_) => 3.1,
        FeatureType::Tree(_) => 4.0,
        FeatureType::Rock => 2.5,
    }
}

// #[cfg(test)]
// mod renderer_tests {
//     use super::*;
//     use crate::components::terrain::TerrainChunkComponent;
//     use std::collections::HashMap;

//     mod visual_updates {
//         use super::*;

//         #[test]
//         fn test_sprite_updates_on_chunk_change() {
//             let mut app = App::new();
//             app.add_plugins(MinimalPlugins)
//                 .add_plugins(AssetPlugin::default())
//                 .add_plugins(TerrainRenderPlugin);

//             // Setup initial state
//             let test_chunk = setup_test_chunk();
//             let test_assets = setup_test_assets();
//             app.insert_resource(create_test_render_config());
//             app.insert_resource(test_assets.clone());

//             // Spawn initial chunk
//             let chunk_entity = app.world_mut().spawn(test_chunk.clone()).id();

//             // Run initial update
//             app.update();

//             // Modify chunk data
//             let new_biome = BiomeType::Forest;
//             if let Some(mut chunk) = app
//                 .world_mut()
//                 .get_mut::<TerrainChunkComponent>(chunk_entity)
//             {
//                 for biome in chunk.biome_map.iter_mut() {
//                     *biome = new_biome;
//                 }
//             }

//             // Run update after modification
//             app.update();

//             let mut query = app.world_mut().query::<(&Sprite, &Transform)>();
//             assert!(verify_visual_state(&test_chunk, query.iter(&app.world())));

//             // Verify sprites updated
//             let mut query = app.world_mut().query::<(&Sprite, &Parent)>();
//             for (sprite, parent) in query.iter(&app.world_mut()) {
//                 if parent.get() == chunk_entity {
//                     if let Some(atlas) = &sprite.texture_atlas {
//                         assert_eq!(
//                             atlas.index, test_assets.tile_mappings[&new_biome],
//                             "Sprite index should match new biome"
//                         );
//                     }
//                 }
//             }
//         }

//         #[test]
//         fn test_height_based_scaling() {
//             let mut app = App::new();
//             app.add_plugins(MinimalPlugins)
//                 .add_plugins(AssetPlugin::default())
//                 .add_plugins(TerrainRenderPlugin);

//             // Setup chunk with varying heights
//             let mut test_chunk = setup_test_chunk();
//             let chunk_size = (test_chunk.height_map.len() as f32).sqrt() as usize;

//             // Create a height gradient
//             for y in 0..chunk_size {
//                 for x in 0..chunk_size {
//                     let idx = y * chunk_size + x;
//                     test_chunk.height_map[idx] = (x as f32 / chunk_size as f32);
//                 }
//             }

//             app.insert_resource(create_test_render_config());
//             app.insert_resource(setup_test_assets());

//             let chunk_for_verification = test_chunk.clone();

//             // Spawn chunk
//             app.world_mut().spawn(test_chunk);
//             app.update();

//             let mut query = app.world_mut().query::<(&Sprite, &Transform)>();
//             assert!(
//                 verify_visual_state(&chunk_for_verification, query.iter(&app.world())),
//                 "Visual state verification failed"
//             );

//             // Verify scaling
//             let mut query = app
//                 .world_mut()
//                 .query::<(&Transform, &TerrainTileComponent)>();
//             for (transform, tile) in query.iter(&app.world_mut()) {
//                 let expected_height = tile.position.x / (chunk_size as f32 * 16.0); // normalize by chunk width
//                 let expected_scale = 1.0 + (expected_height * 0.2);
//                 assert!(
//                     (transform.scale.x - expected_scale).abs() < 0.01,
//                     "Scale should match height-based calculation"
//                 );
//             }
//         }

//         #[test]
//         fn test_comprehensive_visual_state() {
//             let mut app = App::new();
//             app.add_plugins(MinimalPlugins)
//                 .add_plugins(AssetPlugin::default())
//                 .add_plugins(TerrainRenderPlugin);

//             // Setup chunk with varied terrain
//             let mut test_chunk = setup_test_chunk();
//             let chunk_size = (test_chunk.height_map.len() as f32).sqrt() as usize;

//             // Create interesting patterns for testing
//             for y in 0..chunk_size {
//                 for x in 0..chunk_size {
//                     let idx = y * chunk_size + x;
//                     // Vary height
//                     test_chunk.height_map[idx] =
//                         (x as f32 / chunk_size as f32).sin() * (y as f32 / chunk_size as f32).cos();
//                     // Vary moisture
//                     test_chunk.moisture_map[idx] =
//                         (x as f32 / chunk_size as f32).cos() * (y as f32 / chunk_size as f32).sin();
//                     // Vary biomes based on height and moisture
//                     test_chunk.biome_map[idx] = if test_chunk.height_map[idx] < 0.3 {
//                         BiomeType::Water
//                     } else if test_chunk.moisture_map[idx] > 0.7 {
//                         BiomeType::Forest
//                     } else {
//                         BiomeType::Grass
//                     };
//                 }
//             }

//             app.insert_resource(create_test_render_config());
//             app.insert_resource(setup_test_assets());

//             // Spawn chunk
//             app.world_mut().spawn(test_chunk.clone());
//             app.update();

//             // Verify complete visual state
//             let mut sprite_query = app.world_mut().query::<(&Sprite, &Transform)>();
//             assert!(verify_visual_state(
//                 &test_chunk,
//                 sprite_query.iter(&app.world())
//             ));
//         }
//     }

//     mod layer_management {
//         use super::*;

//         #[test]
//         fn test_biome_layer_ordering() {
//             let mut app = App::new();
//             app.add_plugins(MinimalPlugins)
//                 .add_plugins(AssetPlugin::default())
//                 .add_plugins(TerrainRenderPlugin);

//             // Create chunk with all biome types
//             let mut test_chunk = setup_test_chunk();
//             let biomes = vec![
//                 BiomeType::Water,
//                 BiomeType::Sand,
//                 BiomeType::Grass,
//                 BiomeType::Forest,
//                 BiomeType::Crops,
//                 BiomeType::Orchard,
//             ];

//             // Set different biomes in different areas
//             let chunk_size = (test_chunk.height_map.len() as f32).sqrt() as usize;
//             for (i, biome) in biomes.iter().enumerate() {
//                 let start_y = (i * chunk_size) / biomes.len();
//                 let end_y = ((i + 1) * chunk_size) / biomes.len();

//                 for y in start_y..end_y {
//                     for x in 0..chunk_size {
//                         test_chunk.biome_map[y * chunk_size + x] = *biome;
//                     }
//                 }
//             }

//             app.insert_resource(create_test_render_config());
//             app.insert_resource(setup_test_assets());
//             app.world_mut().spawn(test_chunk);
//             app.update();

//             // Verify z-ordering
//             let mut query = app
//                 .world_mut()
//                 .query::<(&Transform, &TerrainTileComponent)>();
//             for (transform, tile) in query.iter(&app.world_mut()) {
//                 let expected_z = get_biome_layer(tile.biome_type);
//                 assert_eq!(
//                     transform.translation.z, expected_z,
//                     "Z-position should match biome layer"
//                 );
//             }

//             // Verify relative ordering
//             let water_z = get_biome_layer(BiomeType::Water);
//             let land_z = get_biome_layer(BiomeType::Grass);
//             let forest_z = get_biome_layer(BiomeType::Forest);

//             assert!(water_z < land_z, "Water should be below land");
//             assert!(land_z < forest_z, "Land should be below forest");
//         }

//         // #[test]
//         // fn test_feature_layer_offset() {
//         //     let mut app = App::new();
//         //     app.add_plugins(MinimalPlugins)
//         //         .add_plugins(AssetPlugin::default())test
//         //         .add_plugins(TerrainRenderPlugin);

//         //     // Setup terrain with features
//         //     let test_chunk = setup_test_chunk();
//         //     let render_config = TerrainRenderConfig {
//         //         feature_layer_offset: 10.0,
//         //         ..default()
//         //     };
//         //     let test_assets = setup_test_assets();

//         //     app.insert_resource(render_config.clone());
//         //     app.insert_resource(test_assets.clone());

//         //     // Spawn chunk with features
//         //     let chunk_entity = app.world_mut().spawn(test_chunk).id();

//         //     // Spawn some features
//         //     let feature_types = vec![
//         //         FeatureType::Tree(TreeVariant::EvergreenFir),
//         //         FeatureType::Bush(BushVariant::GreenBushel),
//         //         FeatureType::Flower(FlowerVariant::Single),
//         //     ];

//         //     for (i, feature_type) in feature_types.iter().enumerate() {
//         //         let feature = TerrainFeatureComponent {
//         //             feature_type: *feature_type,
//         //             variant: FeatureVariant::Tree(TreeVariant::EvergreenFir),
//         //             position: Vec2::new(i as f32 * 16.0, 0.0),
//         //             rotation: 0.0,
//         //             scale: 1.0,
//         //         };

//         //         app.world_mut().spawn(feature).set_parent(chunk_entity);
//         //     }

//         //     app.update();

//         //     // Verify feature z-positions
//         //     let mut query = app
//         //         .world_mut()
//         //         .query::<(&Transform, &TerrainFeatureComponent)>();
//         //     for (transform, _feature) in query.iter(&app.world_mut()) {
//         //         assert!(
//         //             transform.translation.z >= render_config.feature_layer_offset,
//         //             "Features should be above terrain layer"
//         //         );

//         //         // Check z-fighting prevention
//         //         let mut query_others = app.world_mut().query::<&Transform>();
//         //         for other_transform in query_others.iter(&app.world()) {
//         //             if other_transform.translation != transform.translation {
//         //                 assert!(
//         //                     (other_transform.translation.z - transform.translation.z).abs() >= 0.01,
//         //                     "Features should have distinct z-positions"
//         //                 );
//         //             }
//         //         }
//         //     }
//         // }
//     }

//     mod chunk_rendering {
//         use super::*;

//         /// Helper function to create a test chunk with known properties
//         #[allow(dead_code)]
//         fn create_test_chunk(position: IVec2, chunk_size: u32) -> TerrainChunkComponent {
//             let mut chunk = TerrainChunkComponent::new(position, chunk_size);

//             // Fill with test data - using consistent height of 0.5
//             let size = (chunk_size * chunk_size) as usize;
//             for i in 0..size {
//                 chunk.height_map[i] = 0.5; // This height value corresponds to scale 1.1
//                 chunk.moisture_map[i] = 0.5;
//                 chunk.biome_map[i] = BiomeType::Grass;
//             }

//             chunk
//         }

//         #[test]
//         fn test_chunk_boundaries() {}
//     }
//     // Helper functions for tests

//     fn create_test_render_config() -> TerrainRenderConfig {
//         TerrainRenderConfig {
//             tile_size: 16.0,
//             feature_layer_offset: 10.0,
//         }
//     }

//     fn setup_test_chunk() -> TerrainChunkComponent {
//         let chunk_size = 32;
//         let mut chunk = TerrainChunkComponent::new(IVec2::new(0, 0), chunk_size);

//         // Fill with test data
//         for y in 0..chunk_size {
//             for x in 0..chunk_size {
//                 let idx = (y * chunk_size + x) as usize;

//                 // Create a recognizable pattern for testing
//                 chunk.height_map[idx] = x as f32 / chunk_size as f32;
//                 chunk.moisture_map[idx] = y as f32 / chunk_size as f32;

//                 // Assign biomes in a checkered pattern
//                 chunk.biome_map[idx] = match (x % 2, y % 2) {
//                     (0, 0) => BiomeType::Grass,
//                     (1, 0) => BiomeType::Forest,
//                     (0, 1) => BiomeType::Water,
//                     (1, 1) => BiomeType::Sand,
//                     _ => BiomeType::Grass,
//                 };
//             }
//         }

//         chunk
//     }

//     fn setup_test_assets() -> TerrainAssets {
//         let mut assets = TerrainAssets::new();

//         // Create dummy texture handles
//         assets.tile_texture = Handle::default();
//         assets.feature_texture = Handle::default();

//         // Create basic texture atlas layout
//         let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 4, 4, None, None);
//         assets.tile_layout = Handle::default();
//         assets.feature_layout = Handle::default();

//         // Setup mappings
//         assets.tile_mappings = HashMap::from([
//             (BiomeType::Grass, 0),
//             (BiomeType::Forest, 1),
//             (BiomeType::Water, 2),
//             (BiomeType::Sand, 3),
//             (BiomeType::Crops, 4),
//             (BiomeType::Orchard, 5),
//         ]);

//         assets.feature_mappings = HashMap::from([
//             (FeatureType::Tree(TreeVariant::EvergreenFir), 0),
//             (FeatureType::Tree(TreeVariant::WiltingFir), 1),
//             (FeatureType::Bush(BushVariant::GreenBushel), 2),
//             (FeatureType::Flower(FlowerVariant::Single), 3),
//         ]);

//         assets
//     }

//     fn verify_visual_state<'a>(
//         chunk: &TerrainChunkComponent,
//         sprites: impl Iterator<Item = (&'a Sprite, &'a Transform)>,
//     ) -> bool {
//         let mut is_valid = true;
//         let chunk_size = (chunk.height_map.len() as f32).sqrt() as u32;
//         let tile_size = 16.0;

//         let chunk_world_offset = Vec2::new(
//             chunk.position.x as f32 * chunk_size as f32 * tile_size,
//             chunk.position.y as f32 * chunk_size as f32 * tile_size,
//         );

//         let mut expected_tiles = HashMap::new();
//         for y in 0..chunk_size {
//             for x in 0..chunk_size {
//                 let idx = (y * chunk_size + x) as usize;
//                 expected_tiles.insert((x, y), (chunk.biome_map[idx], chunk.height_map[idx]));
//             }
//         }

//         println!("Verifying chunk at position: {:?}", chunk.position);
//         println!("Chunk world offset: {:?}", chunk_world_offset);
//         println!("Expected tiles count: {}", expected_tiles.len());

//         let mut sprite_count = 0;
//         for (sprite, transform) in sprites {
//             sprite_count += 1;
//             let relative_pos = transform.translation.xy() - chunk_world_offset;
//             let grid_x = (relative_pos.x / tile_size).floor() as u32;
//             let grid_y = (relative_pos.y / tile_size).floor() as u32;

//             println!("Sprite world pos: {:?}", transform.translation.xy());
//             println!("Relative pos: {:?}", relative_pos);
//             println!("Checking sprite at grid position ({}, {})", grid_x, grid_y);

//             if grid_x < chunk_size && grid_y < chunk_size {
//                 if let Some(&(expected_biome, height)) = expected_tiles.get(&(grid_x, grid_y)) {
//                     // Verify sprite index matches biome
//                     if let Some(atlas) = &sprite.texture_atlas {
//                         let expected_index = match expected_biome {
//                             BiomeType::Forest => 1,
//                             BiomeType::Grass => 0,
//                             BiomeType::Water => 2,
//                             BiomeType::Sand => 3,
//                             BiomeType::Crops => 4,
//                             BiomeType::Orchard => 5,
//                         };

//                         if atlas.index != expected_index {
//                             println!(
//                                 "Sprite index mismatch at ({}, {}): expected {}, got {}",
//                                 grid_x, grid_y, expected_index, atlas.index
//                             );
//                             is_valid = false;
//                         }
//                     }

//                     // Verify scale based on height
//                     let expected_scale = 1.0 + (height * 0.2);
//                     let scale_diff = (transform.scale - Vec3::splat(expected_scale)).length();
//                     if scale_diff >= 0.01 {
//                         println!(
//                             "Scale mismatch at ({}, {}): expected {} (height: {}), got {:?}",
//                             grid_x, grid_y, expected_scale, height, transform.scale
//                         );
//                         is_valid = false;
//                     }

//                     // Verify z-position based on biome layer
//                     let expected_z = get_biome_layer(expected_biome);
//                     if transform.translation.z < 0.0 || transform.translation.z > expected_z + 0.1 {
//                         println!(
//                             "Invalid z-position at ({}, {}): z={}, expected <= {}",
//                             grid_x,
//                             grid_y,
//                             transform.translation.z,
//                             expected_z + 0.1
//                         );
//                         is_valid = false;
//                     }

//                     // Verify world position
//                     let expected_world_pos = chunk_world_offset
//                         + Vec2::new(grid_x as f32 * tile_size, grid_y as f32 * tile_size);
//                     let pos_diff = (transform.translation.xy() - expected_world_pos).length();
//                     if pos_diff >= 0.01 {
//                         println!(
//                             "Position mismatch at ({}, {}): expected {:?}, got {:?}, diff={}",
//                             grid_x,
//                             grid_y,
//                             expected_world_pos,
//                             transform.translation.xy(),
//                             pos_diff
//                         );
//                         is_valid = false;
//                     }
//                 } else {
//                     println!("Tile data not found for position ({}, {})", grid_x, grid_y);
//                     is_valid = false;
//                 }
//             } else {
//                 println!(
//                     "Sprite position ({}, {}) outside chunk bounds",
//                     grid_x, grid_y
//                 );
//                 // Don't set is_valid to false here as the sprite might belong to another chunk
//             }
//         }

//         println!("Total sprites checked: {}", sprite_count);
//         println!("Verification result: {}", is_valid);

//         if !is_valid {
//             println!("Chunk verification failed:");
//             println!("  Position: {:?}", chunk.position);
//             println!("  World offset: {:?}", chunk_world_offset);
//             println!("  Chunk size: {}", chunk_size);
//             println!("  Total tiles: {}", chunk.height_map.len());
//         }

//         is_valid
//     }
// }
