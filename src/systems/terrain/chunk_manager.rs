use bevy::prelude::*;
use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashSet;

use crate::components::terrain::*;
use crate::resources::terrain::{TerrainAssets, TerrainConfig, TerrainState};
use crate::systems::terrain::{
    generator::{generate_chunk_data, try_spawn_feature},
    TerrainGeneratorSystem,
};

#[derive(Component)]
pub struct ComputeChunk {
    pub task: Task<CommandQueue>,
    pub position: IVec2,
    pub started_at: std::time::Instant,
}

/// Resource to track chunk loading state
#[derive(Resource)]
pub struct ChunkLoadingState {
    pub chunks_to_load: HashSet<IVec2>,
    pub chunks_to_unload: HashSet<IVec2>,
    pub loading_radius: i32,
    pub max_chunks_per_frame: usize,
    last_update: std::time::Instant,
    last_scale: f32,
}

impl Default for ChunkLoadingState {
    fn default() -> Self {
        Self {
            chunks_to_load: HashSet::new(),
            chunks_to_unload: HashSet::new(),
            loading_radius: 5,
            max_chunks_per_frame: 80,
            last_update: std::time::Instant::now(),
            last_scale: 1.0,
        }
    }
}

fn should_update_chunks(loading_state: &ChunkLoadingState, current_scale: f32) -> bool {
    const MIN_UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);
    const SCALE_THRESHOLD: f32 = 0.1;

    let time_since_update = loading_state.last_update.elapsed();
    let scale_change = (current_scale - loading_state.last_scale).abs();

    info!(
        "Update check: time_since_update={:?}, scale_change={}, current_scale={}, last_scale={}",
        time_since_update, scale_change, current_scale, loading_state.last_scale
    );

    time_since_update > MIN_UPDATE_INTERVAL || scale_change > SCALE_THRESHOLD
}

fn get_camera_center_chunk(
    camera_query: &Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
    terrain_state: &TerrainState,
) -> IVec2 {
    if let Ok((transform, _)) = camera_query.get_single() {
        let camera_pos = transform.translation.truncate();
        let chunk_size_world = terrain_state.chunk_size as f32 * terrain_state.scale;
        IVec2::new(
            (camera_pos.x / chunk_size_world).round() as i32,
            (camera_pos.y / chunk_size_world).round() as i32,
        )
    } else {
        IVec2::ZERO
    }
}

/// System to update which chunks should be loaded/unloaded
pub fn update_chunk_tracking_system(
    camera_query: Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
    terrain_state: Res<TerrainState>,
    mut loading_state: ResMut<ChunkLoadingState>,
    chunks: Query<(Entity, &TerrainChunkComponent)>,
) {
    // Add hysteresis to prevent thrashing during zoom
    let view_scale = if let Ok((_, projection)) = camera_query.get_single() {
        projection.scale
    } else {
        return;
    };

    info!("Current view scale: {}", view_scale);

    // Only update chunks if we've moved significantly or zoomed significantly
    if !should_update_chunks(&loading_state, view_scale) {
        info!("Skipped update due to should_update_chunks check");
        return;
    }

    info!("Proceeding with chunk update");

    let visible_chunks = get_visible_chunks(&camera_query, &terrain_state);
    let current_chunks: HashSet<_> = chunks.iter().map(|(_, c)| c.position).collect();
    let center_chunk = get_camera_center_chunk(&camera_query, &terrain_state);

    // Prepare chunks to load, sorted by distance
    let mut chunks_to_load: Vec<_> = visible_chunks
        .difference(&current_chunks)
        .copied()
        .collect();

    chunks_to_load.sort_by_key(|&pos| {
        let diff = pos - center_chunk;
        (diff.x * diff.x + diff.y * diff.y) as u32
    });

    loading_state.chunks_to_load = chunks_to_load.into_iter().collect();

    let unload_margin = (view_scale * 1.5).ceil() as i32; // Keep more chunks loaded during zoom
    loading_state.chunks_to_unload = current_chunks
        .difference(&visible_chunks)
        .filter(|&pos| {
            let diff = *pos - center_chunk;
            diff.x.abs() > unload_margin || diff.y.abs() > unload_margin
        })
        .copied()
        .collect();

    // Update the scale for the next frames
    loading_state.last_scale = view_scale;
    loading_state.last_update = std::time::Instant::now();
}

/// System to update active chunks list in terrain state
pub fn update_active_chunks_system(
    chunks: Query<&TerrainChunkComponent>,
    mut terrain_state: ResMut<TerrainState>,
) {
    terrain_state.active_chunks = chunks.iter().map(|c| c.position).collect();
}

fn get_visible_chunks(
    camera_query: &Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
    terrain_state: &TerrainState,
) -> HashSet<IVec2> {
    let mut visible = HashSet::new();

    if let Ok((camera_transform, projection)) = camera_query.get_single() {
        let camera_pos = camera_transform.translation.truncate();
        let chunk_size_world = terrain_state.chunk_size as f32 * terrain_state.scale;
        let window_chunks_x = (800.0 * projection.scale / chunk_size_world).ceil() as i32;
        let window_chunks_y = (600.0 * projection.scale / chunk_size_world).ceil() as i32;
        let chunks_to_load = (window_chunks_x.max(window_chunks_y) * 2).max(10);

        info!(
            "Visibility calc: camera_pos={:?}, chunk_size_world={}, window_chunks=({},{}), chunks_to_load={}",
            camera_pos, chunk_size_world, window_chunks_x, window_chunks_y, chunks_to_load
        );

        let center_chunk = IVec2::new(
            (camera_pos.x / chunk_size_world).round() as i32,
            (camera_pos.y / chunk_size_world).round() as i32,
        );

        info!("Center chunk: {:?}", center_chunk);

        // Log the chunk range we're about to process
        info!(
            "Chunk range: x=({} to {}), y=({} to {})",
            center_chunk.x - chunks_to_load,
            center_chunk.x + chunks_to_load,
            center_chunk.y - chunks_to_load,
            center_chunk.y + chunks_to_load
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

pub fn spawn_chunk_tasks(mut commands: Commands, mut loading_state: ResMut<ChunkLoadingState>) {
    let thread_pool = AsyncComputeTaskPool::get();

    let chunks_to_load: Vec<_> = loading_state
        .chunks_to_load
        .iter()
        .take(loading_state.max_chunks_per_frame)
        .copied()
        .collect();

    info!("Attempting to spawn {} chunk tasks", chunks_to_load.len());

    for &chunk_pos in chunks_to_load.iter() {
        info!("Spawning task for chunk at {:?}", chunk_pos);
        let entity = commands.spawn_empty().id();

        let task = thread_pool.spawn(async move {
            info!("Starting chunk generation for {:?}", chunk_pos);
            let mut command_queue = CommandQueue::default();

            command_queue.push(move |world: &mut World| {
                // Create a SystemState to access our resources
                let mut system_state = SystemState::<(
                    Res<TerrainState>,
                    Res<TerrainConfig>,
                    Res<TerrainAssets>,
                    ResMut<TerrainGeneratorSystem>,
                )>::new(world);

                let (terrain_state, terrain_config, terrain_assets, mut generator) =
                    system_state.get_mut(world);

                // Create and generate the chunk
                let mut chunk = TerrainChunkComponent::new(chunk_pos, terrain_state.chunk_size);
                generate_chunk_data(&mut chunk, &terrain_state, &terrain_config, &mut generator);

                let chunk_size = terrain_state.chunk_size as usize;
                let chunk_world_pos =
                    chunk.world_position(terrain_state.chunk_size, terrain_state.scale);
                let mut rng = StdRng::seed_from_u64(terrain_state.seed);

                // Collect all feature data first
                let mut features_data = Vec::new();
                for y in 0..chunk_size {
                    for x in 0..chunk_size {
                        let idx = y * chunk_size + x;
                        let biome = chunk.biome_map[idx];
                        let world_pos = Vec2::new(
                            chunk_world_pos.x + x as f32 * terrain_state.scale,
                            chunk_world_pos.y + y as f32 * terrain_state.scale,
                        );

                        if let Some(feature) =
                            try_spawn_feature(world_pos, biome, &terrain_config, &mut rng)
                        {
                            if let Some(&sprite_index) =
                                terrain_assets.feature_mappings.get(&feature.feature_type)
                            {
                                features_data.push((
                                    feature.clone(),
                                    Sprite::from_atlas_image(
                                        terrain_assets.feature_texture.clone(),
                                        TextureAtlas {
                                            layout: terrain_assets.feature_layout.clone(),
                                            index: sprite_index,
                                        },
                                    ),
                                    Transform::from_translation(feature.position.extend(10.0))
                                        .with_rotation(Quat::from_rotation_z(feature.rotation))
                                        .with_scale(Vec3::splat(feature.scale)),
                                ));
                            }
                        }
                    }
                }

                // Generate tile data
                let mut tile_data = Vec::new();
                for y in 0..chunk_size {
                    for x in 0..chunk_size {
                        let idx = y * chunk_size + x;
                        let tile_world_pos = chunk_world_pos
                            + Vec2::new(
                                x as f32 * terrain_state.scale,
                                y as f32 * terrain_state.scale,
                            );

                        let biome = chunk.biome_map[idx];
                        if let Some(&sprite_index) = terrain_assets.tile_mappings.get(&biome) {
                            tile_data.push((
                                TerrainTileComponent {
                                    position: tile_world_pos,
                                    biome_type: biome,
                                    sprite_index,
                                },
                                Sprite::from_atlas_image(
                                    terrain_assets.tile_texture.clone(),
                                    TextureAtlas {
                                        layout: terrain_assets.tile_layout.clone(),
                                        index: sprite_index,
                                    },
                                ),
                                Transform::from_translation(tile_world_pos.extend(0.0))
                                    .with_scale(Vec3::splat(1.0)),
                            ));
                        }
                    }
                }

                // Release resources before world manipulation
                system_state.apply(world);

                // Spawn the chunk entity with all children at once
                world
                    .entity_mut(entity)
                    .insert((
                        chunk,
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                    ))
                    .with_children(|builder| {
                        // Spawn all terrain tiles as children
                        for (tile, sprite, transform) in tile_data {
                            builder.spawn((
                                tile,
                                sprite,
                                transform,
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ));
                        }

                        // Spawn all features as children
                        for (feature, sprite, transform) in features_data {
                            builder.spawn((
                                feature,
                                sprite,
                                transform,
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                            ));
                        }
                    });
            });

            info!("Completed chunk generation for {:?}", chunk_pos);
            command_queue
        });

        commands.entity(entity).insert(ComputeChunk {
            task,
            position: chunk_pos,
            started_at: std::time::Instant::now(),
        });

        loading_state.chunks_to_load.remove(&chunk_pos);
    }
}

pub fn handle_chunk_tasks(
    mut commands: Commands,
    mut compute_chunks: Query<(Entity, &mut ComputeChunk)>,
    chunks: Query<&TerrainChunkComponent>,
) {
    for (entity, mut compute_chunk) in &mut compute_chunks {
        // First check if this chunk position is already loaded
        if chunks.iter().any(|c| c.position == compute_chunk.position) {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        if let Some(mut command_queue) = block_on(future::poll_once(&mut compute_chunk.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).remove::<ComputeChunk>();
        }
    }
}

/// System to handle chunk unloading
pub fn chunk_unloading_system(
    mut commands: Commands,
    mut loading_state: ResMut<ChunkLoadingState>,
    chunks: Query<(Entity, &TerrainChunkComponent)>,
) {
    let to_unload: Vec<_> = loading_state.chunks_to_unload.iter().copied().collect();

    // Unload chunks that are no longer needed
    for position in to_unload {
        if let Some((entity, _)) = chunks.iter().find(|(_, chunk)| chunk.position == position) {
            commands.entity(entity).despawn_recursive();
        }
        loading_state.chunks_to_unload.remove(&position);
    }
}

pub fn cleanup_stale_tasks(mut commands: Commands, compute_chunks: Query<(Entity, &ComputeChunk)>) {
    const TASK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

    for (entity, compute_chunk) in compute_chunks.iter() {
        if compute_chunk.started_at.elapsed() > TASK_TIMEOUT {
            commands.entity(entity).remove::<ComputeChunk>();
            // Optionally requeue the chunk for later processing
            info!(
                "Removed stale compute task for chunk at {:?}",
                compute_chunk.position
            );
        }
    }
}

#[derive(Component)]
pub struct DebugMarker;

#[allow(dead_code)]
fn debug_chunk_metrics(
    compute_chunks: Query<&ComputeChunk>,
    chunks: Query<&TerrainChunkComponent>,
    loading_state: Res<ChunkLoadingState>,
    time: Res<Time>,
) {
    // Log every 5 seconds
    if time.elapsed_secs() % 5.0 < 0.1 {
        info!(
            "Terrain Metrics:\n\
            - Active compute tasks: {}\n\
            - Spawned chunks: {}\n\
            - Chunks queued for loading: {}\n\
            - Chunks queued for unloading: {}\n",
            compute_chunks.iter().count(),
            chunks.iter().count(),
            loading_state.chunks_to_load.len(),
            loading_state.chunks_to_unload.len(),
        );

        // Log tasks that have been running for too long
        for compute_chunk in compute_chunks.iter() {
            let duration = compute_chunk.started_at.elapsed();
            if duration.as_secs() > 5 {
                warn!(
                    "Long-running task detected! Chunk {:?} has been computing for {:?}",
                    compute_chunk.position, duration
                );
            }
        }
    }
}

use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ChunkStats {
    generated_chunks: HashMap<IVec2, std::time::Instant>,
    total_generations: usize,
}

#[allow(dead_code)]
fn track_chunk_lifecycle(
    mut stats: ResMut<ChunkStats>,
    chunks: Query<&TerrainChunkComponent, Added<TerrainChunkComponent>>,
) {
    for chunk in chunks.iter() {
        if let Some(last_gen) = stats.generated_chunks.get(&chunk.position) {
            info!(
                "Chunk at {:?} regenerated after {} seconds",
                chunk.position,
                last_gen.elapsed().as_secs_f32()
            );
        }
        stats
            .generated_chunks
            .insert(chunk.position, std::time::Instant::now());
        stats.total_generations += 1;
    }
}

pub fn cleanup_orphaned_components(
    mut commands: Commands,
    orphaned_features: Query<Entity, (With<TerrainFeatureComponent>, Without<Parent>)>,
    orphaned_tiles: Query<Entity, (With<TerrainTileComponent>, Without<Parent>)>,
) {
    for entity in orphaned_features.iter() {
        warn!("Despawning orphaned feature");
        commands.entity(entity).despawn();
    }

    for entity in orphaned_tiles.iter() {
        warn!("Despawning orphaned tile");
        commands.entity(entity).despawn();
    }
}

pub fn verify_chunk_cleanup(
    chunks: Query<&TerrainChunkComponent>,
    features: Query<&TerrainFeatureComponent>,
    tiles: Query<&TerrainTileComponent>,
    loading_state: Res<ChunkLoadingState>,
) {
    // Log counts every frame where we have unloads pending
    if !loading_state.chunks_to_unload.is_empty() {
        info!(
            "After unload - Chunks: {}, Features: {}, Tiles: {}",
            chunks.iter().count(),
            features.iter().count(),
            tiles.iter().count()
        );

        // Check if any chunks that should be unloaded still exist
        for chunk in chunks.iter() {
            if loading_state.chunks_to_unload.contains(&chunk.position) {
                warn!(
                    "Found chunk that should have been unloaded: {:?}",
                    chunk.position
                );
            }
        }
    }
}

pub fn verify_chunk_hierarchy(
    chunks: Query<(Entity, &TerrainChunkComponent, &Children)>,
    orphaned_features: Query<Entity, (With<TerrainFeatureComponent>, Without<Parent>)>,
    orphaned_tiles: Query<Entity, (With<TerrainTileComponent>, Without<Parent>)>,
    mut commands: Commands,
) {
    // Clean up any orphaned entities
    for entity in orphaned_features.iter().chain(orphaned_tiles.iter()) {
        commands.entity(entity).despawn();
    }

    // Verify all chunks have expected children
    for (chunk_entity, chunk, children) in chunks.iter() {
        if children.is_empty() {
            warn!(
                "Chunk at {:?} has no children - cleaning up",
                chunk.position
            );
            commands.entity(chunk_entity).despawn_recursive();
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
                spawn_chunk_tasks,
                handle_chunk_tasks,
                chunk_unloading_system,
                update_active_chunks_system,
                cleanup_stale_tasks,
                cleanup_orphaned_components,
                verify_chunk_cleanup,
                verify_chunk_hierarchy,
            )
                .chain(),
        );
    }
}
