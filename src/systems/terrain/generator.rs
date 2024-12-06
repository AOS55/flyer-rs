use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

use crate::components::terrain::*;
use crate::resources::terrain::config::BiomeConfig;
use crate::resources::terrain::{TerrainAssets, TerrainConfig, TerrainState};
use crate::systems::terrain::noise::NoiseGenerator;
use crate::systems::terrain::rivers::generate_rivers;

#[derive(Resource)]
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
}

pub fn terrain_generation_system(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut TerrainChunkComponent), Added<TerrainChunkComponent>>,
    terrain_state: Res<TerrainState>,
    terrain_config: Res<TerrainConfig>,
    terrain_assets: Res<TerrainAssets>,
    mut generator: ResMut<TerrainGeneratorSystem>,
) {
    if !chunks.is_empty() {
        info!("Generating chunks: {}", chunks.iter().count());
    }

    for (entity, mut chunk) in chunks.iter_mut() {
        generate_chunk_data(&mut chunk, &terrain_state, &terrain_config, &mut generator);
        spawn_chunk_features(
            &mut commands,
            entity,
            &chunk,
            &terrain_state,
            &terrain_config,
            &terrain_assets,
            &mut generator.rng,
        );
    }
}

pub fn generate_chunk_data(
    chunk: &mut TerrainChunkComponent,
    state: &TerrainState,
    config: &TerrainConfig,
    generator: &mut TerrainGeneratorSystem,
) {
    let chunk_size = state.chunk_size as i32;
    let chunk_world_pos = chunk.world_position(state.chunk_size, state.scale);

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let idx = (y * chunk_size + x) as usize;
            let world_pos = Vec2::new(
                chunk_world_pos.x + x as f32 * state.scale,
                chunk_world_pos.y + y as f32 * state.scale,
            );

            // Generate base terrain values
            chunk.height_map[idx] = generator.get_height(world_pos);
            chunk.moisture_map[idx] = generator.get_moisture(world_pos);

            // Determine biome
            chunk.biome_map[idx] = determine_biome(
                chunk.height_map[idx],
                chunk.moisture_map[idx],
                generator.get_river_value(world_pos),
                generator.get_detail_value(world_pos),
                &config.biome,
            );
        }
    }
    generate_rivers(chunk, state, generator, &config.noise.river);

    smooth_biome_transitions(chunk, chunk_size);
}

fn determine_biome(
    height: f32,
    moisture: f32,
    river_value: f32,
    detail_value: f32,
    config: &BiomeConfig,
) -> BiomeType {
    if height < config.thresholds.water {
        return BiomeType::Water;
    }

    let water_proximity = (height - config.thresholds.water).abs();
    let water_moisture = (1.0 - water_proximity.min(0.2) * 5.0).max(0.0);
    let adjusted_moisture = (moisture + water_moisture) * 0.7; // Reduced moisture influence

    let beach_factor = smoothstep(
        config.thresholds.water,
        config.thresholds.water + config.thresholds.beach_width,
        height,
    );
    if beach_factor > 0.0 && beach_factor < 0.8 {
        return BiomeType::Sand;
    }

    // Increased forest threshold and added height influence
    let forest_threshold = lerp(
        config.thresholds.forest_moisture + 0.1,
        config.thresholds.forest_moisture + 0.7,
        height,
    );

    if adjusted_moisture > forest_threshold && detail_value > 0.3 {
        BiomeType::Forest
    } else {
        BiomeType::Grass
    }
}

// Smoothstep function for smooth transitions
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// Linear interpolation helper
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn spawn_chunk_features(
    commands: &mut Commands,
    chunk_entity: Entity,
    chunk: &TerrainChunkComponent,
    state: &TerrainState,
    config: &TerrainConfig,
    assets: &TerrainAssets,
    rng: &mut StdRng,
) {
    let chunk_size = state.chunk_size as usize;
    let chunk_world_pos = chunk.world_position(state.chunk_size, state.scale);

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let idx = y * chunk_size + x;
            let biome = chunk.biome_map[idx];
            let world_pos = Vec2::new(
                chunk_world_pos.x + x as f32 * state.scale,
                chunk_world_pos.y + y as f32 * state.scale,
            );

            // Try to spawn features based on biome and config
            for (feature_type, base_density) in &config.feature.densities {
                let multiplier = config
                    .feature
                    .biome_multipliers
                    .get(&(feature_type.clone(), biome))
                    .unwrap_or(&1.0);

                if rng.gen::<f32>() < base_density * multiplier {
                    // Call `try_spawn_feature` and check if it returns `Some`
                    if let Some(feature) = try_spawn_feature(world_pos, biome, config, rng) {
                        // Only pass `feature` to `spawn_feature` if it exists
                        spawn_feature(commands, chunk_entity, feature, assets);
                    }
                }
            }
        }
    }
}

fn smooth_biome_transitions(chunk: &mut TerrainChunkComponent, chunk_size: i32) {
    let mut smoothed = chunk.biome_map.clone();
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
                        *biome_counts
                            .entry(chunk.biome_map[neighbor_idx])
                            .or_insert(0.0) += weight;
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

    chunk.biome_map = smoothed;
}

fn distance_to_water(chunk: &TerrainChunkComponent, x: i32, y: i32, chunk_size: i32) -> f32 {
    let mut min_distance = f32::MAX;

    for dy in -1..=1 {
        for dx in -1..=1 {
            let nx = x + dx;
            let ny = y + dy;

            if nx >= 0 && nx < chunk_size && ny >= 0 && ny < chunk_size {
                let idx = (ny * chunk_size + nx) as usize;
                if chunk.biome_map[idx] == BiomeType::Water {
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    if distance < min_distance {
                        min_distance = distance;
                    }
                }
            }
        }
    }

    min_distance
}

fn try_spawn_feature(
    pos: Vec2,
    biome: BiomeType,
    config: &TerrainConfig,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    match biome {
        BiomeType::Forest => try_spawn_tree(pos, config, rng),
        BiomeType::Orchard => try_spawn_orchard_tree(pos, config, rng),
        BiomeType::Crops => try_spawn_bush(pos, config, rng),
        BiomeType::Grass => try_spawn_flower(pos, config, rng),
        _ => None,
    }
}

fn spawn_feature(
    commands: &mut Commands,
    chunk_entity: Entity,
    feature: TerrainFeatureComponent,
    assets: &TerrainAssets,
) {
    if let Some(&sprite_index) = assets.feature_mappings.get(&feature.feature_type) {
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
                Transform::from_translation(feature.position.extend(10.0))
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

// Implement similar functions for other feature types
fn try_spawn_tree(
    pos: Vec2,
    config: &TerrainConfig,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    // Determine tree type based on random value
    let tree_type = if rng.gen::<f32>() < 0.6 {
        TreeVariant::EvergreenFir
    } else {
        TreeVariant::WiltingFir
    };

    let feature_type = FeatureType::Tree(tree_type);
    let density = *config.feature.densities.get(&feature_type).unwrap_or(&0.0);

    if rng.gen::<f32>() > density {
        return None;
    }

    Some(TerrainFeatureComponent {
        feature_type,
        variant: FeatureVariant::Tree(tree_type),
        position: pos,
        rotation: std::f32::consts::TAU,
        scale: 0.8 + rng.gen::<f32>() * 0.4,
    })
}

fn try_spawn_orchard_tree(
    pos: Vec2,
    config: &TerrainConfig,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    let object_probability = rng.gen::<f32>();

    let tree_type = if rng.gen::<f32>() < 0.75 {
        TreeVariant::AppleTree
    } else {
        TreeVariant::PrunedTree
    };

    let feature_type = FeatureType::Tree(tree_type);
    let density = *config.feature.densities.get(&feature_type).unwrap_or(&0.0);

    if object_probability > density {
        return None;
    }

    Some(TerrainFeatureComponent {
        feature_type,
        variant: FeatureVariant::Tree(tree_type),
        position: pos,
        rotation: std::f32::consts::TAU,
        scale: 0.9 + rng.gen::<f32>() * 0.2,
    })
}

fn try_spawn_bush(
    pos: Vec2,
    _config: &TerrainConfig,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    let spawn_chance = rng.gen::<f32>();
    if spawn_chance > 0.3 {
        // Adjust this value as needed
        return None;
    }

    let bush_type = match rng.gen_range(0..3) {
        0 => BushVariant::GreenBushel,
        1 => BushVariant::RipeBushel,
        _ => BushVariant::DeadBushel,
    };

    let feature_type = FeatureType::Bush(bush_type);

    Some(TerrainFeatureComponent {
        feature_type,
        variant: FeatureVariant::Bush(bush_type),
        position: pos,
        rotation: std::f32::consts::TAU,
        scale: 0.7 + rng.gen::<f32>() * 0.3,
    })
}

fn try_spawn_flower(
    pos: Vec2,
    config: &TerrainConfig,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    // Using original orchard flower logic

    let flower_type = match rng.gen_range(0..4) {
        0 => FlowerVariant::Single,
        1 => FlowerVariant::Double,
        2 => FlowerVariant::Quad,
        _ => FlowerVariant::Cluster,
    };

    let feature_type = FeatureType::Flower(flower_type);
    let density = *config.feature.densities.get(&feature_type).unwrap_or(&0.0);

    if rng.gen::<f32>() > density {
        return None;
    }

    Some(TerrainFeatureComponent {
        feature_type,
        variant: FeatureVariant::Flower(flower_type),
        position: pos,
        rotation: std::f32::consts::TAU,
        scale: 0.6 + rng.gen::<f32>() * 0.2,
    })
}
