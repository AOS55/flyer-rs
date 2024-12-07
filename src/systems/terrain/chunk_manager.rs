use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use std::collections::HashSet;

use crate::components::terrain::*;
use crate::resources::terrain::{TerrainAssets, TerrainState};

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
            max_chunks_per_frame: 8,
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

    // Only update chunks if we've moved significantly or zoomed significantly
    if !should_update_chunks(&loading_state, view_scale) {
        return;
    }

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

    // Update the scale for the next frame
    loading_state.last_scale = view_scale;
    loading_state.last_update = std::time::Instant::now();
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

pub fn spawn_chunk_tasks(mut commands: Commands, mut loading_state: ResMut<ChunkLoadingState>) {
    let thread_pool = AsyncComputeTaskPool::get();

    let chunks_to_load: Vec<_> = loading_state
        .chunks_to_load
        .iter()
        .take(loading_state.max_chunks_per_frame)
        .copied()
        .collect();

    for &chunk_pos in chunks_to_load.iter() {
        let entity = commands.spawn_empty().id();

        let task = thread_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();

            command_queue.push(move |world: &mut World| {
                // Create a SystemState to properly access our resources
                let mut system_state =
                    SystemState::<(Res<TerrainState>, Res<TerrainAssets>)>::new(world);

                let (terrain_state, terrain_assets) = system_state.get_mut(world);

                let chunk_size = terrain_state.chunk_size;
                let tile_size = terrain_state.scale;
                let chunk_world_pos = Vec2::new(
                    chunk_pos.x as f32 * chunk_size as f32 * tile_size,
                    chunk_pos.y as f32 * chunk_size as f32 * tile_size,
                );

                // First, prepare all tile data
                let mut tile_data = Vec::new();
                for y in 0..chunk_size as usize {
                    for x in 0..chunk_size as usize {
                        let tile_world_pos = chunk_world_pos
                            + Vec2::new(
                                x as f32 * terrain_state.scale,
                                y as f32 * terrain_state.scale,
                            );

                        tile_data.push((
                            Sprite {
                                image: terrain_assets.tile_texture.clone(),
                                texture_atlas: Some(TextureAtlas {
                                    layout: terrain_assets.tile_layout.clone(),
                                    index: 0,
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
                        ));
                    }
                }

                // Release the system state borrow
                system_state.apply(world);

                // Now spawn the chunk entity with its components
                let mut chunk_entity = world.entity_mut(entity);
                chunk_entity.insert((
                    TerrainChunkComponent::new(chunk_pos, chunk_size),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));

                // Spawn all tiles and add them as children
                for tile_components in tile_data {
                    let tile_entity = world.spawn(tile_components).id();
                    world.entity_mut(entity).add_child(tile_entity);
                }
            });

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
) {
    for (entity, mut compute_chunk) in &mut compute_chunks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut compute_chunk.task)) {
            // Execute the command queue which will create our chunk
            commands.append(&mut command_queue);

            // Remove the compute task component as we're done
            commands.entity(entity).remove::<ComputeChunk>();
        }
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

pub fn debug_check_system(
    compute_chunks: Query<(Entity, &ComputeChunk)>,
    orphaned_sprites: Query<Entity, (With<Sprite>, Without<Parent>)>,
    time: Res<Time>,
) {
    if time.elapsed_secs() % 5.0 < 0.1 {
        // Run every 5 seconds
        info!(
            "Debug status:\n\
            - Pending compute tasks: {}\n\
            - Orphaned sprites: {}",
            compute_chunks.iter().count(),
            orphaned_sprites.iter().count()
        );
    }
}

use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ChunkStats {
    generated_chunks: HashMap<IVec2, std::time::Instant>,
    total_generations: usize,
}

pub fn track_chunk_lifecycle(
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
                debug_check_system,
                track_chunk_lifecycle,
            )
                .chain(),
        );
    }
}
