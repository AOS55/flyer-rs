use bevy::prelude::*;
use std::collections::HashSet;

use crate::components::terrain::*;
use crate::resources::terrain::{TerrainAssets, TerrainState};

/// Resource to track chunk loading state
#[derive(Resource)]
pub struct ChunkLoadingState {
    pub chunks_to_load: HashSet<IVec2>,
    pub chunks_to_unload: HashSet<IVec2>,
    pub loading_radius: i32,
    pub max_chunks_per_frame: usize,
}

impl Default for ChunkLoadingState {
    fn default() -> Self {
        Self {
            chunks_to_load: HashSet::new(),
            chunks_to_unload: HashSet::new(),
            loading_radius: 5,
            max_chunks_per_frame: 8,
        }
    }
}

/// System to update which chunks should be loaded/unloaded
pub fn update_chunk_tracking_system(
    camera_query: Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
    terrain_state: Res<TerrainState>,
    mut loading_state: ResMut<ChunkLoadingState>,
    chunks: Query<(Entity, &TerrainChunkComponent)>,
) {
    let visible_chunks = get_visible_chunks(&camera_query, &terrain_state);
    let current_chunks: HashSet<_> = chunks.iter().map(|(_, c)| c.position).collect();

    // Determine chunks to load and unload
    loading_state.chunks_to_load = visible_chunks
        .difference(&current_chunks)
        .copied()
        .collect();

    loading_state.chunks_to_unload = current_chunks
        .difference(&visible_chunks)
        .copied()
        .collect();

    // Logging for debugging
    // info!("Chunks to load: {:?}", loading_state.chunks_to_load);
    // info!("Chunks to unload: {:?}", loading_state.chunks_to_unload);
}

/// System to handle chunk loading
pub fn chunk_loading_system(
    mut commands: Commands,
    mut loading_state: ResMut<ChunkLoadingState>,
    terrain_state: Res<TerrainState>,
    terrain_assets: Res<TerrainAssets>,
) {
    // Load new chunks
    let chunks_to_load: Vec<_> = loading_state
        .chunks_to_load
        .iter()
        .take(loading_state.max_chunks_per_frame)
        .copied()
        .collect();

    for &chunk_pos in &chunks_to_load {
        spawn_chunk(&mut commands, chunk_pos, &terrain_state, &terrain_assets);
        loading_state.chunks_to_load.remove(&chunk_pos);

        // Logging for debugging
        // info!(
        //     "Attempting to spawn chunk at world coordinates: {}",
        //     Vec2::new(
        //         chunk_pos.x as f32 * terrain_state.chunk_size as f32 * terrain_state.scale,
        //         chunk_pos.y as f32 * terrain_state.chunk_size as f32 * terrain_state.scale
        //     )
        // );
        // info!("Spawning chunk at position: {:?}", chunk_pos);
    }
}

/// System to handle chunk unloading
pub fn chunk_unloading_system(
    mut commands: Commands,
    mut loading_state: ResMut<ChunkLoadingState>,
    chunks: Query<(Entity, &TerrainChunkComponent)>,
) {
    // Unload chunks that are no longer needed
    for (entity, chunk) in chunks.iter() {
        if loading_state.chunks_to_unload.contains(&chunk.position) {
            commands.entity(entity).despawn_recursive();
            loading_state.chunks_to_unload.remove(&chunk.position);
        }
    }
}

/// System to update active chunks list in terrain state
pub fn update_active_chunks_system(
    chunks: Query<&TerrainChunkComponent>,
    mut terrain_state: ResMut<TerrainState>,
) {
    terrain_state.active_chunks = chunks.iter().map(|c| c.position).collect();

    // Logging for debugging
    // info!("Active chunks: {:?}", terrain_state.active_chunks);
}

fn get_visible_chunks(
    camera_query: &Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
    terrain_state: &TerrainState,
) -> HashSet<IVec2> {
    let mut visible = HashSet::new();

    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_pos = camera_transform.translation.truncate();
        // let view_distance = projection.scale * 3.0;
        let chunk_size_world = terrain_state.chunk_size as f32 * terrain_state.scale;
        let window_chunks_x = (800.0 * projection.scale / chunk_size_world).ceil() as i32;
        let window_chunks_y = (600.0 * projection.scale / chunk_size_world).ceil() as i32;
        let chunks_to_load = (window_chunks_x.max(window_chunks_y) * 2).max(10);

        let center_chunk = IVec2::new(
            (camera_pos.x / chunk_size_world).round() as i32,
            (camera_pos.y / chunk_size_world).round() as i32,
        );

        for x in -chunks_to_load..=chunks_to_load {
            for y in -chunks_to_load..=chunks_to_load {
                let chunk_pos = center_chunk + IVec2::new(x, y);
                visible.insert(chunk_pos);
            }
        }
    }

    visible
}

/// Helper function to spawn a new chunk
fn spawn_chunk(
    commands: &mut Commands,
    position: IVec2,
    terrain_state: &TerrainState,
    terrain_assets: &TerrainAssets,
) {
    // Create the main chunk entity
    let chunk_entity = commands
        .spawn((
            TerrainChunkComponent::new(position, terrain_state.chunk_size),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    // Logging for debugging
    // info!("Spawning chunk at position: {:?}", position);

    // Spawn tile entities as children
    let chunk_size = terrain_state.chunk_size as usize;
    let tile_size = terrain_state.scale;

    let chunk_world_pos = Vec2::new(
        position.x as f32 * chunk_size as f32 * tile_size,
        position.y as f32 * chunk_size as f32 * tile_size,
    );

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let tile_world_pos = chunk_world_pos
                + Vec2::new(
                    x as f32 * terrain_state.scale,
                    y as f32 * terrain_state.scale,
                );

            // println!(
            //     "Spawning tile at local pos ({}, {}) world pos: {:?}",
            //     x, y, tile_world_pos
            // );

            commands
                .spawn((
                    Sprite {
                        image: terrain_assets.tile_texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: terrain_assets.tile_layout.clone(),
                            index: 0, // Default tile
                        }),
                        ..default()
                    },
                    TerrainTileComponent {
                        biome_type: BiomeType::Grass,
                        position: tile_world_pos,
                        sprite_index: 0,
                    },
                    Transform::from_translation(tile_world_pos.extend(0.0))
                        .with_scale(Vec3::splat(1.0)),
                ))
                .set_parent(chunk_entity);

            // Logging for debugging
            // info!("Spawning tile at position: {:?}", tile_world_pos);
        }
    }
}

/// Plugin to organize chunk management systems
pub struct ChunkManagerPlugin;

impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkLoadingState>().add_systems(
            Update,
            (
                update_chunk_tracking_system,
                chunk_loading_system,
                chunk_unloading_system,
                update_active_chunks_system,
            )
                .chain(),
        );
    }
}

// #[cfg(test)]
// mod chunk_tests {
//     use super::*;
//     use crate::components::terrain::TerrainChunkComponent;
//     use crate::systems::terrain::{generator::generate_chunk_data, TerrainGeneratorSystem};
//     use std::collections::HashMap;

//     /// Helper function to create a test chunk
//     fn create_test_chunk(position: IVec2, chunk_size: u32) -> TerrainChunkComponent {
//         TerrainChunkComponent {
//             position,
//             height_map: vec![0.0; (chunk_size * chunk_size) as usize],
//             moisture_map: vec![0.0; (chunk_size * chunk_size) as usize],
//             biome_map: vec![BiomeType::Grass; (chunk_size * chunk_size) as usize],
//         }
//     }

//     /// Helper function to create test state
//     fn create_test_state(chunk_size: u32, scale: f32, seed: u64) -> TerrainState {
//         TerrainState {
//             chunk_size,
//             scale,
//             seed,
//             ..Default::default()
//         }
//     }

//     #[test]
//     fn test_chunk_size_consistency() {
//         let chunk_size = 32;
//         let mut chunk = create_test_chunk(IVec2::ZERO, chunk_size);
//         let state = create_test_state(chunk_size, 1.0, 12345);
//         let config = TerrainGeneratorConfig::default();
//         let terrain_config = TerrainConfig::default();
//         let mut generator = TerrainGeneratorSystem::new(12345, &config);

//         generate_chunk_data(&mut chunk, &state, &terrain_config, &mut generator);

//         assert_eq!(chunk.height_map.len(), (chunk_size * chunk_size) as usize);
//         assert_eq!(chunk.moisture_map.len(), (chunk_size * chunk_size) as usize);
//         assert_eq!(chunk.biome_map.len(), (chunk_size * chunk_size) as usize);
//     }

//     #[test]
//     fn test_chunk_horizontal_continuity() {
//         let chunk_size = 32;
//         let state = create_test_state(chunk_size, 1.0, 12345);
//         let config = TerrainGeneratorConfig::default();
//         let terrain_config = TerrainConfig::default();
//         let mut generator = TerrainGeneratorSystem::new(12345, &config);

//         // Create two adjacent chunks
//         let mut chunk1 = create_test_chunk(IVec2::new(0, 0), chunk_size);
//         let mut chunk2 = create_test_chunk(IVec2::new(1, 0), chunk_size);

//         generate_chunk_data(&mut chunk1, &state, &terrain_config, &mut generator);
//         generate_chunk_data(&mut chunk2, &state, &terrain_config, &mut generator);

//         for y in 0..chunk_size as usize {
//             let chunk1_idx = y * chunk_size as usize + (chunk_size as usize - 1);
//             let chunk2_idx = y * chunk_size as usize;

//             let height1 = chunk1.height_map[chunk1_idx];
//             let height2 = chunk2.height_map[chunk2_idx];
//             let diff = (height1 - height2).abs();

//             // Use a more forgiving epsilon for floating point comparison
//             const EPSILON: f32 = 1e-6;
//             assert!(
//                 diff < EPSILON,
//                 "Height mismatch at y={}: chunk1={}, chunk2={}, diff={}",
//                 y,
//                 height1,
//                 height2,
//                 diff
//             );
//         }
//     }

//     #[test]
//     fn test_chunk_vertical_continuity() {
//         let chunk_size = 32;
//         let state = TerrainState {
//             chunk_size,
//             scale: 1.0,
//             seed: 12345,
//             ..Default::default()
//         };

//         let terrain_config = TerrainConfig {
//             noise_scale: 100.0,
//             noise_octaves: 4,
//             noise_persistence: 0.5,
//             noise_lacunarity: 2.0,
//             ..Default::default()
//         };
//         let config = TerrainGeneratorConfig::default();

//         let mut generator = TerrainGeneratorSystem::new(12345, &config);

//         let mut chunk1 = create_test_chunk(IVec2::new(0, 0), chunk_size);
//         let mut chunk2 = create_test_chunk(IVec2::new(0, 1), chunk_size);

//         generate_chunk_data(&mut chunk1, &state, &terrain_config, &mut generator);
//         generate_chunk_data(&mut chunk2, &state, &terrain_config, &mut generator);
//         for x in 0..chunk_size as usize {
//             let chunk1_idx = (chunk_size as usize - 1) * chunk_size as usize + x; // Top row of bottom chunk
//             let chunk2_idx = x; // Bottom row of top chunk

//             let height1 = chunk1.height_map[chunk1_idx];
//             let height2 = chunk2.height_map[chunk2_idx];

//             assert!(
//                 (height1 - height2).abs() < 1e-6,
//                 "Height mismatch at x={}: chunk1={}, chunk2={}, diff={}",
//                 x,
//                 height1,
//                 height2,
//                 (height1 - height2).abs()
//             );
//         }
//     }

//     #[test]
//     fn test_chunk_diagonal_continuity() {
//         let chunk_size = 32;
//         let state = TerrainState {
//             chunk_size,
//             scale: 1.0,
//             seed: 12345,
//             ..Default::default()
//         };

//         let terrain_config = TerrainConfig {
//             noise_scale: 100.0,
//             noise_octaves: 4,
//             noise_persistence: 0.5,
//             noise_lacunarity: 2.0,
//             ..Default::default()
//         };
//         let config = TerrainGeneratorConfig::default();
//         let mut generator = TerrainGeneratorSystem::new(12345, &config);

//         // Generate four chunks in a 2x2 grid
//         let mut chunks = vec![
//             (
//                 UVec2::new(0, 0),
//                 create_test_chunk(IVec2::new(0, 0), chunk_size),
//             ),
//             (
//                 UVec2::new(1, 0),
//                 create_test_chunk(IVec2::new(1, 0), chunk_size),
//             ),
//             (
//                 UVec2::new(0, 1),
//                 create_test_chunk(IVec2::new(0, 1), chunk_size),
//             ),
//             (
//                 UVec2::new(1, 1),
//                 create_test_chunk(IVec2::new(1, 1), chunk_size),
//             ),
//         ];

//         for (_, chunk) in chunks.iter_mut() {
//             generate_chunk_data(chunk, &state, &terrain_config, &mut generator);
//         }

//         // Check corner where all four chunks meet
//         let heights = vec![
//             chunks[0].1.height_map
//                 [(chunk_size as usize - 1) * chunk_size as usize + (chunk_size as usize - 1)], // Top-right of bottom-left chunk
//             chunks[1].1.height_map[(chunk_size as usize - 1) * chunk_size as usize], // Top-left of bottom-right chunk
//             chunks[2].1.height_map[chunk_size as usize - 1], // Bottom-right of top-left chunk
//             chunks[3].1.height_map[0],                       // Bottom-left of top-right chunk
//         ];

//         // All corners should match
//         for i in 1..heights.len() {
//             assert!(
//                 (heights[0] - heights[i]).abs() < 1e-6,
//                 "Corner height mismatch: {} vs {}, diff: {}",
//                 heights[0],
//                 heights[i],
//                 (heights[0] - heights[i]).abs()
//             );
//         }
//     }

//     #[test]
//     fn test_chunk_value_ranges() {
//         let chunk_size = 32;
//         let mut chunk = create_test_chunk(IVec2::ZERO, chunk_size);
//         let state = create_test_state(chunk_size, 1.0, 12345);
//         let terrain_config = TerrainConfig::default();
//         let config = TerrainGeneratorConfig::default();
//         let mut generator = TerrainGeneratorSystem::new(12345, &config);

//         generate_chunk_data(&mut chunk, &state, &terrain_config, &mut generator);

//         // Check value ranges
//         for i in 0..chunk.height_map.len() {
//             // Height should be between 0 and 1
//             assert!(chunk.height_map[i] >= 0.0 && chunk.height_map[i] <= 1.0);
//             // Moisture should be between 0 and 1
//             assert!(chunk.moisture_map[i] >= 0.0 && chunk.moisture_map[i] <= 1.0);
//             // Biome should be valid
//             assert!(matches!(
//                 chunk.biome_map[i],
//                 BiomeType::Grass
//                     | BiomeType::Forest
//                     | BiomeType::Water
//                     | BiomeType::Sand
//                     | BiomeType::Crops
//                     | BiomeType::Orchard
//             ));
//         }
//     }

//     #[test]
//     fn test_chunk_position_influence() {
//         let chunk_size = 32;
//         let state = create_test_state(chunk_size, 1.0, 12345);
//         let terrain_config = TerrainConfig::default();
//         let config = TerrainGeneratorConfig::default();
//         let mut generator = TerrainGeneratorSystem::new(12345, &config);

//         // Generate chunks at different positions
//         let mut chunk1 = create_test_chunk(IVec2::new(0, 0), chunk_size);
//         let mut chunk2 = create_test_chunk(IVec2::new(100, 100), chunk_size);

//         generate_chunk_data(&mut chunk1, &state, &terrain_config, &mut generator);
//         generate_chunk_data(&mut chunk2, &state, &terrain_config, &mut generator);

//         // Chunks should be different (extremely unlikely to be identical)
//         assert_ne!(chunk1.height_map, chunk2.height_map);
//         assert_ne!(chunk1.moisture_map, chunk2.moisture_map);
//     }

//     #[test]
//     fn test_scale_influences() {
//         let chunk_size = 32;
//         let config = TerrainGeneratorConfig::default();
//         let mut generator = TerrainGeneratorSystem::new(12345, &config);

//         // Test world scale influence
//         let state1 = TerrainState {
//             chunk_size,
//             scale: 1.0,
//             ..Default::default()
//         };
//         let state2 = TerrainState {
//             chunk_size,
//             scale: 4.0,
//             ..Default::default()
//         };
//         let config = TerrainConfig::default();

//         let mut chunk1 = create_test_chunk(IVec2::ZERO, chunk_size);
//         let mut chunk2 = create_test_chunk(IVec2::ZERO, chunk_size);

//         generate_chunk_data(&mut chunk1, &state1, &config, &mut generator);
//         generate_chunk_data(&mut chunk2, &state2, &config, &mut generator);

//         // World scale should affect final positions but not height values
//         assert_eq!(chunk1.height_map, chunk2.height_map);

//         // Test noise scale influence
//         let config1 = TerrainConfig {
//             noise_scale: 100.0,
//             ..Default::default()
//         };
//         let config2 = TerrainConfig {
//             noise_scale: 200.0,
//             ..Default::default()
//         };
//         let state = TerrainState::default();

//         generate_chunk_data(&mut chunk1, &state, &config1, &mut generator);
//         generate_chunk_data(&mut chunk2, &state, &config2, &mut generator);

//         // Different noise scales should produce different height patterns
//         assert_ne!(chunk1.height_map, chunk2.height_map);
//     }

//     #[test]
//     fn test_biome_distribution() {
//         let chunk_size = 32;
//         let mut chunk = create_test_chunk(IVec2::ZERO, chunk_size);
//         let state = create_test_state(chunk_size, 1.0, 12345);
//         let config = TerrainGeneratorConfig::default();
//         let terrain_config = TerrainConfig::default();
//         let mut generator = TerrainGeneratorSystem::new(12345, &config);

//         generate_chunk_data(&mut chunk, &state, &terrain_config, &mut generator);

//         // Count biomes
//         let mut biome_counts: HashMap<BiomeType, usize> = HashMap::new();
//         for &biome in &chunk.biome_map {
//             *biome_counts.entry(biome).or_insert(0) += 1;
//         }

//         // There should be some variety in biomes
//         assert!(
//             biome_counts.len() > 1,
//             "Should have more than one biome type"
//         );

//         // No single biome should dominate completely (shouldn't be more than 80% of tiles)
//         let total_tiles = chunk_size * chunk_size;
//         // for &count in biome_counts.values() {
//         //     assert!(count as f32 / total_tiles as f32 < 0.8);
//         // }
//     }
// }
