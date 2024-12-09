use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::components::terrain::*;
use crate::resources::terrain::config::BiomeConfig;
use crate::resources::terrain::BiomeFeatureConfig;
use crate::resources::terrain::{TerrainConfig, TerrainState};
use crate::systems::terrain::noise::NoiseGenerator;
// use crate::systems::terrain::rivers::generate_rivers;

#[derive(Resource, Clone)]
pub struct TerrainGeneratorSystem {
    height_generator: NoiseGenerator,
    moisture_generator: NoiseGenerator,
    river_generator: NoiseGenerator,
    detail_generator: NoiseGenerator,
    rng: StdRng,
}

impl TerrainGeneratorSystem {
    pub fn new(state: &TerrainState, config: &TerrainConfig) -> Self {
        let mut height_generator = NoiseGenerator::new(state.seed);
        let mut moisture_generator = NoiseGenerator::new(state.seed.wrapping_add(1));
        let mut river_generator = NoiseGenerator::new(state.seed.wrapping_add(2));
        let detail_generator = NoiseGenerator::new(state.seed.wrapping_add(3));

        // Configure generators based on config
        height_generator.set_value_range(0.0, 1.0);
        for layer in &config.noise.height.layers {
            height_generator.add_layer(layer.clone());
        }

        moisture_generator.set_value_range(0.0, 1.0);
        for layer in &config.noise.moisture.layers {
            moisture_generator.add_layer(layer.clone());
        }

        river_generator.set_value_range(0.0, config.noise.river.max_length);

        Self {
            height_generator,
            moisture_generator,
            river_generator,
            detail_generator,
            rng: StdRng::seed_from_u64(state.seed),
        }
    }

    pub fn generate_chunk(
        &mut self,
        position: IVec2,
        state: &TerrainState,
        config: &TerrainConfig,
    ) -> TerrainChunkComponent {
        // Initialize result structures
        let mut result = TerrainChunkComponent::new(position, state);

        self.generate_base_terrain(state, &mut result);

        self.generate_biomes(state, &mut result, config);

        self.generate_features(state, &mut result, config);

        result
    }

    fn generate_base_terrain(&self, state: &TerrainState, result: &mut TerrainChunkComponent) {
        for y in 0..state.chunk_size {
            for x in 0..state.chunk_size {
                let idx = y * state.chunk_size + x;
                let world_pos = state.get_tile_world_pos(result.position, x, y);
                result.height_map[idx as usize] = self.get_height(world_pos);
                result.moisture_map[idx as usize] = self.get_moisture(world_pos);
            }
        }
    }

    fn generate_biomes(
        &self,
        state: &TerrainState,
        result: &mut TerrainChunkComponent,
        config: &TerrainConfig,
    ) {
        for y in 0..state.chunk_size {
            for x in 0..state.chunk_size {
                let idx = y * state.chunk_size + x;
                let world_pos = state.get_tile_world_pos(result.position, x, y);

                result.biome_map[idx] = determine_biome(
                    result.height_map[idx],
                    result.moisture_map[idx],
                    &config.biome,
                    state.seed,
                    world_pos,
                );
            }
        }
        self.smooth_biome_transitions(&mut result.biome_map, state.chunk_size as i32);
    }

    fn generate_features(
        &mut self,
        state: &TerrainState,
        result: &mut TerrainChunkComponent,
        config: &TerrainConfig,
    ) {
        for y in 0..state.chunk_size {
            for x in 0..state.chunk_size {
                let idx = y * state.chunk_size + x;
                let world_pos = state.get_tile_world_pos(result.position, x, y);

                if let Some(feature) =
                    try_spawn_feature(world_pos, result.biome_map[idx], config, &mut self.rng)
                {
                    result.features.insert(idx, feature);
                }
            }
        }
    }

    pub fn get_height(&self, pos: Vec2) -> f32 {
        self.height_generator.get_noise(pos)
    }

    pub fn get_moisture(&self, pos: Vec2) -> f32 {
        self.moisture_generator.get_noise(pos)
    }

    pub fn get_river_value(&self, pos: Vec2) -> f32 {
        self.river_generator.get_noise(pos)
    }

    pub fn get_detail_value(&self, pos: Vec2) -> f32 {
        self.detail_generator.get_noise(pos)
    }

    fn smooth_biome_transitions(&self, biome_map: &mut Vec<BiomeType>, chunk_size: i32) {
        let mut smoothed = biome_map.clone();
        let kernel_size = 2;

        for y in 0..chunk_size {
            for x in 0..chunk_size {
                let idx = (y * chunk_size + x) as usize;
                let mut biome_counts = HashMap::new();

                for dy in -kernel_size..=kernel_size {
                    for dx in -kernel_size..=kernel_size {
                        let nx = x + dx;
                        let ny = y + dy;

                        if nx >= 0 && nx < chunk_size && ny >= 0 && ny < chunk_size {
                            let weight = 1.0 / ((dx * dx + dy * dy) as f32 + 1.0);
                            let neighbor_idx = (ny * chunk_size + nx) as usize;
                            *biome_counts.entry(biome_map[neighbor_idx]).or_insert(0.0) += weight;
                        }
                    }
                }

                if let Some((dominant_biome, _)) = biome_counts
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                {
                    smoothed[idx] = *dominant_biome;
                }
            }
        }

        *biome_map = smoothed;
    }
}

// pub fn terrain_generation_system(
//     mut commands: Commands,
//     mut chunks: Query<(Entity, &mut TerrainChunkComponent), Added<TerrainChunkComponent>>,
//     terrain_state: Res<TerrainState>,
//     terrain_config: Res<TerrainConfig>,
//     terrain_assets: Res<TerrainAssets>,
//     mut generator: ResMut<TerrainGeneratorSystem>,
// ) {
//     if !chunks.is_empty() {
//         info!("Generating chunks: {}", chunks.iter().count());
//     }

//     for (entity, mut chunk) in chunks.iter_mut() {
//         generate_chunk_data(&mut chunk, &terrain_state, &terrain_config, &mut generator);
//         spawn_chunk_features(
//             &mut commands,
//             entity,
//             &chunk,
//             &terrain_state,
//             &terrain_config,
//             &terrain_assets,
//             &mut generator.rng,
//         );
//     }
// }

// pub fn generate_chunk_data(
//     chunk: &mut TerrainChunkComponent,
//     state: &TerrainState,
//     config: &TerrainConfig,
//     generator: &mut TerrainGeneratorSystem,
// ) {
//     let chunk_size = state.chunk_size as i32;
//     let chunk_world_pos = chunk.world_position(state.chunk_size, state.scale);

//     // First generate base terrain values
//     for y in 0..chunk_size {
//         for x in 0..chunk_size {
//             let idx = (y * chunk_size + x) as usize;
//             let world_pos = Vec2::new(
//                 chunk_world_pos.x + x as f32 * state.scale,
//                 chunk_world_pos.y + y as f32 * state.scale,
//             );

//             // Generate base terrain values
//             chunk.height_map[idx] = generator.get_height(world_pos);
//             chunk.moisture_map[idx] = generator.get_moisture(world_pos);
//         }
//     }

//     // Then generate and apply rivers to modify the terrain
//     generate_rivers(chunk, state, generator, &config.noise.river);

//     // Finally, determine biomes based on the modified terrain
//     for y in 0..chunk_size {
//         for x in 0..chunk_size {
//             let idx = (y * chunk_size + x) as usize;
//             let world_pos = Vec2::new(
//                 chunk_world_pos.x + x as f32 * state.scale,
//                 chunk_world_pos.y + y as f32 * state.scale,
//             );

//             // Determine biome using modified height and moisture values
//             chunk.biome_map[idx] = determine_biome(
//                 chunk.height_map[idx],
//                 chunk.moisture_map[idx],
//                 &config.biome,
//                 state.seed,
//                 world_pos,
//             );
//         }
//     }

//     smooth_biome_transitions(chunk, chunk_size);
// }

fn determine_biome(
    height: f32,
    moisture: f32,
    config: &BiomeConfig,
    seed: u64,
    world_pos: Vec2,
) -> BiomeType {
    // 1. Water bodies (below sea level)
    if height < config.thresholds.water {
        return BiomeType::Water;
    }

    let beach_factor = smoothstep(
        config.thresholds.water,
        config.thresholds.water + config.thresholds.beach_width,
        height,
    );
    if beach_factor > 0.0 && beach_factor < 0.8 {
        return BiomeType::Beach;
    }

    // 2. High elevation regions (mountains and snow)
    let mountain_start = 0.8;
    let snow_start = 0.85;
    if height > mountain_start {
        if height > snow_start {
            return BiomeType::Snow;
        }
        return BiomeType::Mountain;
    }

    // 3. Desert regions based on moisture
    if moisture < config.thresholds.desert_moisture {
        return BiomeType::Desert;
    }

    // 4. Wet regions become forests
    if moisture > config.thresholds.forest_moisture {
        return BiomeType::Forest;
    }

    // 5. Mid-elevation terrain based on moisture and strucure
    get_grid_cell(world_pos, seed, config)
}

// Smoothstep function for smooth transitions
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// fn should_be_forest(world_pos: Vec2, seed: u64, moisture: f32, config: &BiomeConfig) -> bool {
//     // Create large, simple organic shapes using a single hash
//     let mut hasher = std::collections::hash_map::DefaultHasher::new();

//     // Use larger scale for forest regions
//     let scaled_pos = world_pos / 400.0; // Large scale variation for bigger forest patches
//     scaled_pos.x.to_bits().hash(&mut hasher);
//     scaled_pos.y.to_bits().hash(&mut hasher);
//     seed.hash(&mut hasher);

//     let forest_noise = hasher.finish() as f32 / u64::MAX as f32;

//     // Simple moisture check - more moisture = more likely to be forest
//     forest_noise < moisture - config.thresholds.forest_moisture
// }

fn get_grid_cell(world_pos: Vec2, seed: u64, config: &BiomeConfig) -> BiomeType {
    // Use different scales for field size variation
    let large_scale = 400.0; // For grouping fields
    let field_sizes = config.thresholds.field_sizes; // Various field sizes

    // Get field group index using large scale noise
    let group_x = (world_pos.x / large_scale).floor() as i32;
    let group_y = (world_pos.y / large_scale).floor() as i32;

    // Generate consistent field size and rotation for this group
    let mut group_hasher = std::collections::hash_map::DefaultHasher::new();
    group_x.hash(&mut group_hasher);
    group_y.hash(&mut group_hasher);
    seed.hash(&mut group_hasher);
    let group_hash = group_hasher.finish();

    // Select field size for this group
    let field_size = field_sizes[(group_hash % 4) as usize];

    // Generate rotation for this group (0, 45, or 90 degrees)
    let rotation = (group_hash % 4) as f32 * std::f32::consts::FRAC_PI_4;

    // Rotate position
    let rotated_pos = Vec2::new(
        world_pos.x * rotation.cos() - world_pos.y * rotation.sin(),
        world_pos.x * rotation.sin() + world_pos.y * rotation.cos(),
    );

    // Get field coordinates
    let field_x = (rotated_pos.x / field_size).floor() as i32;
    let field_y = (rotated_pos.y / field_size).floor() as i32;

    // Generate field type
    let mut field_hasher = std::collections::hash_map::DefaultHasher::new();
    field_x.hash(&mut field_hasher);
    field_y.hash(&mut field_hasher);
    seed.hash(&mut field_hasher);
    let field_hash = field_hasher.finish();

    // More varied field type distribution
    match field_hash % 7 {
        0..=2 => BiomeType::Crops,   // 40% crops
        3..=5 => BiomeType::Grass,   // 40% grass
        6..=7 => BiomeType::Orchard, // 10% orchards
        _ => BiomeType::Grass,       // 0% grass
    }
}

// fn spawn_chunk_features(
//     commands: &mut Commands,
//     chunk_entity: Entity,
//     chunk: &TerrainChunkComponent,
//     state: &TerrainState,
//     config: &TerrainConfig,
//     assets: &TerrainAssets,
//     rng: &mut StdRng,
// ) {
//     let chunk_size = state.chunk_size as usize;
//     let chunk_world_pos = chunk.world_position(state.chunk_size, state.scale);

//     for y in 0..chunk_size {
//         for x in 0..chunk_size {
//             let idx = y * chunk_size + x;
//             let biome = chunk.biome_map[idx];
//             let world_pos = Vec2::new(
//                 chunk_world_pos.x + x as f32 * state.scale,
//                 chunk_world_pos.y + y as f32 * state.scale,
//             );

//             if let Some(feature) = try_spawn_feature(world_pos, biome, config, rng) {
//                 // Only pass `feature` to `spawn_feature` if it exists
//                 spawn_feature(commands, chunk_entity, feature, assets);
//             }
//         }
//     }
// }

// fn spawn_feature(
//     commands: &mut Commands,
//     chunk_entity: Entity,
//     feature: TerrainFeatureComponent,
//     assets: &TerrainAssets,
// ) {
//     if let Some(&sprite_index) = assets.feature_mappings.get(&feature.feature_type) {
//         commands
//             .spawn((
//                 feature.clone(),
//                 Sprite::from_atlas_image(
//                     assets.feature_texture.clone(),
//                     TextureAtlas {
//                         layout: assets.feature_layout.clone(),
//                         index: sprite_index,
//                     },
//                 ),
//                 Transform::from_translation(feature.position.extend(10.0))
//                     .with_rotation(Quat::from_rotation_z(feature.rotation))
//                     .with_scale(Vec3::splat(feature.scale)),
//                 GlobalTransform::default(),
//                 Visibility::default(),
//                 InheritedVisibility::default(),
//                 ViewVisibility::default(),
//             ))
//             .set_parent(chunk_entity);
//     }
// }

pub fn try_spawn_feature(
    pos: Vec2,
    biome: BiomeType,
    config: &TerrainConfig,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    match biome {
        BiomeType::Grass => try_spawn_generic(pos, &config.feature.grass, rng),
        BiomeType::Forest => try_spawn_generic(pos, &config.feature.forest, rng),
        BiomeType::Crops => try_spawn_generic(pos, &config.feature.crops, rng),
        BiomeType::Orchard => try_spawn_generic(pos, &config.feature.orchard, rng),
        BiomeType::Water => try_spawn_generic(pos, &config.feature.water, rng),
        BiomeType::Beach => try_spawn_generic(pos, &config.feature.beach, rng),
        BiomeType::Desert => try_spawn_generic(pos, &config.feature.desert, rng),
        BiomeType::Mountain => try_spawn_generic(pos, &config.feature.mountain, rng),
        BiomeType::Snow => try_spawn_generic(pos, &config.feature.snow, rng),
    }
}

fn try_spawn_generic<T: BiomeFeatureConfig>(
    pos: Vec2,
    config: &T,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    if config.density() < rng.gen::<f32>() {
        return None;
    }

    config
        .select_feature(rng)
        .map(|feature_type| TerrainFeatureComponent {
            feature_type,
            position: pos,
            rotation: std::f32::consts::TAU,
            scale: 1.0,
        })
}
