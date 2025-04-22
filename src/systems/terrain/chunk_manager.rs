use bevy::prelude::*;
use bevy::tasks::futures_lite::future::{block_on, poll_once};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// Import necessary items from bevy_ecs_tilemap
use bevy_ecs_tilemap::prelude::*;

// Your existing project imports (ensure paths are correct)
use crate::components::terrain::TerrainChunkComponent;
use crate::resources::terrain::{TerrainAssets, TerrainConfig, TerrainState};
use crate::systems::terrain::generator::TerrainGeneratorSystem;

/// Represents an asynchronous task for terrain chunk generation.
#[derive(Component)]
pub struct ChunkGenerationTask {
    /// The asynchronous task generating the terrain chunk.
    pub task: Task<TerrainChunkComponent>, // Expecting direct component
    /// The position of the chunk being generated in chunk coordinates.
    pub position: IVec2,
}

// --- ChunkState Enum, ChunkManager Struct, ChunkManager impl, ViewportArea struct ---
// --- (Keep these as they were in the previous version) ---
#[derive(Clone, Debug)]
pub enum ChunkState {
    PendingLoad,
    Loading {
        entity: Entity,
        started_at: Instant,
    },
    Active {
        entity: Entity,
        last_accessed: Instant,
    },
    PendingUnload {
        entity: Entity,
        marked_at: Instant,
    },
    Error,
}
#[derive(Resource, Debug)]
pub struct ChunkManager {
    chunks: HashMap<IVec2, ChunkState>,
    visible_area: ViewportArea,
}
impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            visible_area: ViewportArea {
                center: IVec2::ZERO,
                radius: 5,
            },
        }
    }
}
impl ChunkManager {
    fn update_visible_area(
        &mut self,
        camera_transform: &Transform,
        camera_projection: &OrthographicProjection,
        state: &TerrainState,
    ) {
        let new_area = ViewportArea::from_camera(camera_transform, camera_projection, state);
        if new_area == self.visible_area {
            return;
        }
        for (pos, chunk_state) in self.chunks.iter_mut() {
            match chunk_state {
                ChunkState::Active { entity, .. } if !new_area.contains(*pos) => {
                    *chunk_state = ChunkState::PendingUnload {
                        entity: *entity,
                        marked_at: Instant::now(),
                    };
                }
                ChunkState::PendingUnload { entity, .. } if new_area.contains(*pos) => {
                    *chunk_state = ChunkState::Active {
                        entity: *entity,
                        last_accessed: Instant::now(),
                    };
                }
                _ => {}
            }
        }
        for x in -new_area.radius..=new_area.radius {
            for y in -new_area.radius..=new_area.radius {
                let pos = new_area.center + IVec2::new(x, y);
                if !self.chunks.contains_key(&pos) {
                    self.chunks.insert(pos, ChunkState::PendingLoad);
                }
            }
        }
        self.visible_area = new_area;
    }
    fn get_chunks_to_load(&self) -> Vec<IVec2> {
        self.chunks
            .iter()
            .filter_map(|(pos, state)| matches!(state, ChunkState::PendingLoad).then_some(*pos))
            .collect()
    }
    fn get_chunks_to_unload(&self) -> Vec<(IVec2, Entity)> {
        self.chunks
            .iter()
            .filter_map(|(pos, state)| {
                if let ChunkState::PendingUnload { entity, .. } = state {
                    Some((*pos, *entity))
                } else {
                    None
                }
            })
            .collect()
    }
    fn set_chunk_state(&mut self, pos: IVec2, state: ChunkState) {
        self.chunks.insert(pos, state);
    }
    fn is_task_stale(&self, pos: IVec2, timeout: Duration) -> bool {
        matches!(self.chunks.get(&pos), Some(ChunkState::Loading { started_at, .. }) if started_at.elapsed() > timeout)
    }
    fn remove_chunk(&mut self, pos: IVec2) {
        self.chunks.remove(&pos);
    }
}
#[derive(Clone, Debug, PartialEq)]
struct ViewportArea {
    center: IVec2,
    radius: i32,
}
impl ViewportArea {
    fn from_camera(
        transform: &Transform,
        _projection: &OrthographicProjection,
        state: &TerrainState,
    ) -> Self {
        let pos = transform.translation.truncate();
        let chunk_pos = state.world_to_chunk(pos);
        let radius = state.loading_radius;
        Self {
            center: chunk_pos,
            radius,
        }
    }
    fn contains(&self, pos: IVec2) -> bool {
        let diff = pos - self.center;
        diff.x.abs() <= self.radius && diff.y.abs() <= self.radius
    }
}
// --- End unchanged sections ---

/// System to update the visible chunks based on the camera's position.
fn update_chunks(
    camera_query: Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
    state: Res<TerrainState>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    if let Ok((camera_transform, camera_projection)) = camera_query.get_single() {
        chunk_manager.update_visible_area(camera_transform, camera_projection, &state);
    }
}

/// System to spawn generation tasks for chunks marked as PendingLoad.
fn handle_chunk_loading(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    generator: Res<TerrainGeneratorSystem>,
    config: Res<TerrainConfig>,
    state: Res<TerrainState>,
    assets: Res<TerrainAssets>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let chunks_to_load = chunk_manager.get_chunks_to_load();

    // --- Access state fields *before* the loop/move ---
    let current_chunk_size = state.chunk_size; // usize
    let current_tile_size = state.tile_size; // f32
                                             // --- End access before loop ---

    for pos in chunks_to_load {
        let mut generator_clone = generator.clone(); // Clone for the task
        let config_clone = config.clone(); // Clone for the task
        let state_clone = state.clone(); // Clone specifically for the task

        // Task now directly returns TerrainChunkComponent
        let task = thread_pool.spawn(async move {
            // Use the cloned versions inside the async block
            generator_clone.generate_chunk(pos, &state_clone, &config_clone)
        });

        // Use the values accessed *before* the loop
        let chunk_world_pos = state.chunk_to_world(pos); // Can still use original state Res here
        let chunk_transform = Transform::from_xyz(chunk_world_pos.x, chunk_world_pos.y, 0.0);

        let tile_size_map = TilemapTileSize {
            x: current_tile_size,
            y: current_tile_size,
        };
        let grid_size_map = TilemapGridSize {
            x: current_tile_size,
            y: current_tile_size,
        };
        let map_size = TilemapSize {
            x: current_chunk_size as u32,
            y: current_chunk_size as u32,
        };

        let chunk_entity_id = commands
            .spawn((
                TilemapBundle {
                    grid_size: grid_size_map,
                    size: map_size,
                    storage: TileStorage::empty(map_size),
                    texture: TilemapTexture::Single(assets.tile_texture.clone()),
                    tile_size: tile_size_map,
                    transform: chunk_transform,
                    map_type: TilemapType::Square,
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                ChunkGenerationTask {
                    task,
                    position: pos,
                },
            ))
            .id();

        chunk_manager.set_chunk_state(
            pos,
            ChunkState::Loading {
                entity: chunk_entity_id,
                started_at: Instant::now(),
            },
        );
    }
}

/// System to handle completed chunk generation tasks.
fn handle_chunk_tasks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    terrain_assets: Res<TerrainAssets>,
    terrain_config: Res<TerrainConfig>,
    terrain_state: Res<TerrainState>,
    mut tasks_query: Query<(Entity, &mut ChunkGenerationTask)>,
    mut tile_storage_query: Query<&mut TileStorage>,
) {
    for (entity, mut task) in tasks_query.iter_mut() {
        if let Some(chunk_data) = block_on(poll_once(&mut task.task)) {
            // Direct component
            let chunk_coordinate = task.position;

            if let Ok(mut tile_storage) = tile_storage_query.get_mut(entity) {
                spawn_chunk_entities(
                    &mut commands,
                    entity,
                    chunk_data, // Pass directly
                    &terrain_assets,
                    &terrain_config,
                    &terrain_state,
                    &mut tile_storage,
                );
                commands.entity(entity).insert(Visibility::Visible);
                chunk_manager.set_chunk_state(
                    chunk_coordinate,
                    ChunkState::Active {
                        entity,
                        last_accessed: Instant::now(),
                    },
                );
            } else {
                error!(
                    "TileStorage not found for chunk entity {:?} at {:?} during task handling.",
                    entity, chunk_coordinate
                );
                chunk_manager.set_chunk_state(chunk_coordinate, ChunkState::Error);
                commands.entity(entity).despawn_recursive();
            }
            commands.entity(entity).remove::<ChunkGenerationTask>();
        }
    }
}

/// Spawns tile entities using bevy_ecs_tilemap and feature entities using Sprites.
fn spawn_chunk_entities(
    commands: &mut Commands,
    chunk_entity: Entity,
    chunk_data: TerrainChunkComponent, // Receive owned data
    terrain_assets: &Res<TerrainAssets>,
    terrain_config: &Res<TerrainConfig>,
    terrain_state: &Res<TerrainState>,
    tile_storage: &mut TileStorage, // Still need this for tiles
) {
    let current_chunk_size = terrain_state.chunk_size;
    let current_tile_size = terrain_state.tile_size;
    let map_grid_size = TilemapGridSize {
        x: current_tile_size,
        y: current_tile_size,
    };
    let map_type = TilemapType::Square;

    commands.entity(chunk_entity).with_children(|parent| {
        // --- Spawn Tile Entities (using bevy_ecs_tilemap - KEEP THIS) ---
        for y in 0..current_chunk_size {
            for x in 0..current_chunk_size {
                let tile_index = y * current_chunk_size + x;
                if tile_index >= chunk_data.biome_map.len() {
                    continue;
                } // Bounds check

                let biome = chunk_data.biome_map[tile_index];
                if let Some(texture_index) = terrain_assets.tile_mappings.get(&biome).copied() {
                    let tile_pos = TilePos {
                        x: x as u32,
                        y: y as u32,
                    };
                    // Spawn the tile entity as a child
                    let tile_entity = parent
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(chunk_entity), // Link to parent tilemap
                            texture_index: TileTextureIndex(texture_index as u32),
                            ..Default::default()
                        })
                        .id();
                    // Set the tile in the TileStorage component (on the parent chunk_entity)
                    tile_storage.checked_set(&tile_pos, tile_entity);
                }
            }
        }

        // --- Spawn Feature Entities ---
        for (pos_index, feature) in chunk_data.features.iter() {
            let index_val = *pos_index;
            let x_coord = index_val % current_chunk_size;
            let y_coord = index_val / current_chunk_size;
            if x_coord >= current_chunk_size || y_coord >= current_chunk_size {
                continue;
            } // Bounds check

            let feature_tile_pos = TilePos {
                x: x_coord as u32,
                y: y_coord as u32,
            };

            if let Some(feature_atlas_index) = terrain_assets
                .feature_mappings
                .get(&feature.feature_type)
                .copied()
            {
                let tile_center_world_offset =
                    feature_tile_pos.center_in_world(&map_grid_size, &map_type);

                // MODIFIED: Use the pattern from the old code that worked
                parent.spawn((
                    // Custom data component
                    feature.clone(), // TerrainFeatureComponent
                    // Configured Sprite component instance
                    Sprite::from_atlas_image(
                        terrain_assets.feature_texture.clone(),
                        TextureAtlas {
                            layout: terrain_assets.feature_layout.clone(),
                            index: feature_atlas_index,
                        },
                    ),
                    // Transform component
                    Transform::from_translation(
                        tile_center_world_offset.extend(terrain_config.render.feature_layer_offset),
                    )
                    .with_rotation(Quat::from_rotation_z(feature.rotation))
                    .with_scale(Vec3::splat(feature.scale)),
                    // Other necessary components often added with Transform
                    GlobalTransform::default(),
                    Visibility::Visible, // Make explicitly visible
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));
            }
        }
    }); // End with_children
}

// --- handle_chunk_unloading and cleanup_stale_tasks remain the same ---
fn handle_chunk_unloading(mut commands: Commands, mut chunk_manager: ResMut<ChunkManager>) {
    let chunks_to_unload = chunk_manager.get_chunks_to_unload();
    let mut unloaded_count = 0;

    // Keep track of positions successfully despawned THIS frame
    let mut despawned_positions = Vec::new();

    for (pos, entity) in chunks_to_unload {
        // Iterate over potentially many pending chunks
        if unloaded_count >= 2 {
            break; // Stop despawning more this frame
        }

        if let Some(entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn_recursive();
            // Mark this position as processed for removal from chunk_manager state later
            despawned_positions.push(pos);
            unloaded_count += 1;
        } else {
            // Entity might already be gone for some reason, ensure we remove state too
            despawned_positions.push(pos);
        }
    }

    // Update the chunk manager state only for the chunks actually despawned
    for pos in despawned_positions {
        chunk_manager.remove_chunk(pos); // Removes from the HashMap
    }
}

fn cleanup_stale_tasks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    tasks_query: Query<(Entity, &ChunkGenerationTask)>,
) {
    const TASK_TIMEOUT: Duration = Duration::from_secs(15);
    let mut stale_tasks = Vec::new();
    for (entity, task) in tasks_query.iter() {
        if chunk_manager.is_task_stale(task.position, TASK_TIMEOUT) {
            warn!("Chunk generation task for {:?} timed out.", task.position);
            stale_tasks.push((entity, task.position));
        }
    }
    for (entity, pos) in stale_tasks {
        if let Some(entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn_recursive();
        }
        chunk_manager.remove_chunk(pos);
    }
}

/// Plugin to manage terrain chunks using bevy_ecs_tilemap.
pub struct ChunkManagerPlugin;

impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkManager>()
            .add_systems(
                Update,
                (
                    update_chunks,
                    apply_deferred,
                    handle_chunk_loading,
                    apply_deferred,
                    handle_chunk_tasks,
                    apply_deferred,
                    handle_chunk_unloading,
                )
                    .chain(),
            )
            .add_systems(Last, cleanup_stale_tasks);
    }
}
