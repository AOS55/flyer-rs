use bevy::prelude::*;
use bevy::tasks::futures_lite::future::{block_on, poll_once};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::components::terrain::*;
use crate::resources::terrain::{TerrainAssets, TerrainConfig, TerrainState};
use crate::systems::terrain::generator::TerrainGeneratorSystem;

/// Represents an asynchronous task for terrain chunk generation.
#[derive(Component)]
pub struct ChunkGenerationTask {
    /// The asynchronous task generating the terrain chunk.
    pub task: Task<TerrainChunkComponent>,
    /// The position of the chunk being generated in chunk coordinates.
    pub position: IVec2,
}

/// Represents tile data for individual terrain tiles.
#[derive(Clone, Debug)]
pub struct TileData {
    /// The position of the tile in local space.
    pub position: Vec2,
    /// The biome type of the tile.
    pub biome_type: BiomeType,
}

/// Enum representing the various states a chunk can be in.
// Chunk Management
#[allow(dead_code)] // States are used internally, but not explicitly called, mainly for debugging
#[derive(Clone)]
enum ChunkState {
    /// The chunk is currently being loaded.
    Loading { entity: Entity, started_at: Instant },
    /// The chunk is active and visible.
    Active {
        entity: Entity,
        last_accessed: Instant,
    },
    /// The chunk is marked for unloading.
    PendingUnload { entity: Entity, marked_at: Instant },
}

/// Manages terrain chunks, their states, and visibility.
#[derive(Resource)]
struct ChunkManager {
    /// A map of chunk positions to their respective states.
    chunks: HashMap<IVec2, ChunkState>,
    /// The visible area represented by the camera viewport.
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

/// Represents the visible area in chunk coordinates.
#[derive(Clone)]
struct ViewportArea {
    /// Center of the visible area (in chunk coordinates).
    center: IVec2,
    /// Radius around the center that defines the visible chunks.
    radius: i32,
}

impl ViewportArea {
    /// Computes the visible area based on the camera's position and zoom.
    fn from_camera(
        camera: Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
        state: &TerrainState,
    ) -> Self {
        let (transform, projection) = camera.single();
        let pos = transform.translation.truncate();
        let chunk_pos = state.world_to_chunk(pos);
        let chunk_world_size = state.chunk_world_size();
        let visible_width = projection.area.width() * projection.scale;
        let visible_height = projection.area.height() * projection.scale;

        // Calculate view radius based on zoom and window size
        let radius = (((visible_width.max(visible_height) / chunk_world_size) / 2.0) + 1.0) as i32;

        Self {
            center: chunk_pos,
            radius,
        }
    }

    /// Determines if a chunk position is within the visible area.
    fn contains(&self, pos: IVec2) -> bool {
        let diff = pos - self.center;
        diff.x.abs() <= self.radius && diff.y.abs() <= self.radius
    }
}

impl ChunkManager {
    /// Updates the visible area based on the camera's position.
    fn update_visible_area(
        &mut self,
        camera: Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
        state: &TerrainState,
    ) {
        let new_area = ViewportArea::from_camera(camera, state);

        // Mark chunks outside the visible area for unloading
        for (pos, state) in self.chunks.iter_mut() {
            match state {
                ChunkState::Active { entity, .. } if !new_area.contains(*pos) => {
                    *state = ChunkState::PendingUnload {
                        entity: *entity,
                        marked_at: Instant::now(),
                    };
                }
                _ => {}
            }
        }

        // Mark new chunks for loading
        for x in -new_area.radius..=new_area.radius {
            for y in -new_area.radius..=new_area.radius {
                let pos = new_area.center + IVec2::new(x, y);
                if !self.chunks.contains_key(&pos) {
                    self.chunks.insert(
                        pos,
                        ChunkState::Loading {
                            entity: Entity::PLACEHOLDER,
                            started_at: Instant::now(),
                        },
                    );
                }
            }
        }

        self.visible_area = new_area;
    }

    /// Returns a list of chunk positions that need to be loaded.
    fn get_chunks_to_load(&self) -> Vec<IVec2> {
        self.chunks
            .iter()
            .filter_map(|(pos, state)| matches!(state, ChunkState::Loading { .. }).then_some(*pos))
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

    fn set_loading(&mut self, pos: IVec2, entity: Entity) {
        self.chunks.insert(
            pos,
            ChunkState::Loading {
                entity,
                started_at: Instant::now(),
            },
        );
    }

    /// Marks a chunk as active once it has been successfully loaded.
    fn activate_chunk(&mut self, pos: IVec2, entity: Entity) {
        if let Some(ChunkState::Loading { .. }) = self.chunks.get(&pos) {
            self.chunks.insert(
                pos,
                ChunkState::Active {
                    entity,
                    last_accessed: Instant::now(),
                },
            );
        }
    }

    fn is_task_stale(&self, pos: IVec2, timeout: Duration) -> bool {
        match self.chunks.get(&pos) {
            Some(ChunkState::Loading { started_at, .. }) => started_at.elapsed() > timeout,
            _ => false,
        }
    }

    /// Removes a chunk from the manager.
    fn remove_chunk(&mut self, pos: IVec2) {
        self.chunks.remove(&pos);
    }
}

// System to update the visible chunks based on the camera's position.
fn update_chunks(
    camera: Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
    state: Res<TerrainState>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    chunk_manager.update_visible_area(camera, &state);
}

/// System to clean up chunks marked for unloading.
fn handle_chunk_loading(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    generator: Res<TerrainGeneratorSystem>,
    config: Res<TerrainConfig>,
    state: Res<TerrainState>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for pos in chunk_manager.get_chunks_to_load() {
        let entity = commands.spawn_empty().id();

        let mut generator = generator.clone();
        let config = config.clone();
        let state = state.clone();

        let task = thread_pool.spawn(async move { generator.generate_chunk(pos, &state, &config) });

        commands.entity(entity).insert(ChunkGenerationTask {
            task,
            position: pos,
        });
        chunk_manager.set_loading(pos, entity);
    }
}

fn handle_chunk_tasks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut tasks: Query<(Entity, &mut ChunkGenerationTask)>,
    terrain_assets: Res<TerrainAssets>,
    terrain_config: Res<TerrainConfig>,
    terrain_state: Res<TerrainState>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(generation_result) = block_on(poll_once(&mut task.task)) {
            let chunk_world_pos = terrain_state.chunk_to_world(task.position);
            commands.entity(entity).insert((
                Transform::from_translation(chunk_world_pos.extend(0.0)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
            spawn_chunk_entities(
                &mut commands,
                entity,
                generation_result,
                &terrain_state,
                &terrain_assets,
                &terrain_config,
            );
            chunk_manager.activate_chunk(task.position, entity);
            commands.entity(entity).remove::<ChunkGenerationTask>();
        }
    }
}

fn spawn_chunk_entities(
    commands: &mut Commands,
    chunk_entity: Entity,
    chunk: TerrainChunkComponent,
    state: &TerrainState,
    assets: &TerrainAssets,
    config: &TerrainConfig,
) {
    // Spawn terrain tiles
    for y in 0..state.chunk_size {
        for x in 0..state.chunk_size {
            let idx = (y * state.chunk_size + x) as usize;
            // let world_pos = state.get_tile_world_pos(chunk.position, x as usize, y as usize);
            let local_pos = Vec2::new(x as f32 * state.tile_size, y as f32 * state.tile_size);
            let biome_type = chunk.biome_map[idx];
            // info!("Biome type: {:?}", biome_type);
            if let Some(&sprite_index) = assets.tile_mappings.get(&biome_type) {
                commands
                    .spawn((
                        Sprite::from_atlas_image(
                            assets.tile_texture.clone(),
                            TextureAtlas {
                                layout: assets.tile_layout.clone(),
                                index: sprite_index,
                            },
                        ),
                        Transform::from_translation(local_pos.extend(0.0)),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        TerrainTileComponent {
                            biome_type,
                            position: local_pos,
                            sprite_index,
                        },
                    ))
                    .set_parent(chunk_entity);
            }
        }
    }

    // Spawn features
    for (idx, feature) in &chunk.features {
        if let Some(&sprite_index) = assets.feature_mappings.get(&feature.feature_type) {
            let world_pos = state.tile_index_to_chunk(*idx);
            commands
                .spawn((
                    feature.clone(),
                    Sprite::from_atlas_image(
                        assets.feature_texture.clone(),
                        TextureAtlas {
                            layout: assets.feature_layout.clone(),
                            index: sprite_index,
                        },
                    ),
                    Transform::from_translation(
                        world_pos.extend(config.render.feature_layer_offset),
                    )
                    .with_rotation(Quat::from_rotation_z(feature.rotation))
                    .with_scale(Vec3::splat(feature.scale)),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .set_parent(chunk_entity);
        }
    }
}

fn handle_chunk_unloading(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    chunks_query: Query<Entity>,
) {
    let chunks_to_unload = chunk_manager.get_chunks_to_unload();

    for (pos, entity) in chunks_to_unload {
        // Verify entity exists before attempting to despawn
        if chunks_query.get(entity).is_ok() {
            commands.entity(entity).despawn_recursive();
            chunk_manager.remove_chunk(pos);
        } else {
            // If entity doesn't exist but is still in manager, clean it up
            warn!("Chunk at {:?} had invalid entity reference", pos);
            chunk_manager.remove_chunk(pos);
        }
    }
}

fn cleanup_stale_tasks(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    tasks: Query<(Entity, &ChunkGenerationTask)>,
) {
    const TASK_TIMEOUT: Duration = Duration::from_secs(5);

    for (entity, task) in &tasks {
        if chunk_manager.is_task_stale(task.position, TASK_TIMEOUT) {
            commands.entity(entity).despawn();
            chunk_manager.remove_chunk(task.position);
        }
    }
}

/// Plugin to manage terrain chunks.
pub struct ChunkManagerPlugin;

impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkManager>()
            .add_systems(
                Update,
                (
                    update_chunks,
                    handle_chunk_loading,
                    handle_chunk_tasks,
                    handle_chunk_unloading,
                )
                    .chain(),
            )
            .add_systems(Last, cleanup_stale_tasks);
    }
}
