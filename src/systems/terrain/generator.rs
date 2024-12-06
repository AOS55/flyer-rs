use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

use crate::components::terrain::*;
use crate::systems::terrain::noise::{NoiseGenerator, NoiseLayer};
use crate::systems::terrain::rivers::generate_rivers;

/// Configuration for each type of noise used in terrain generation
#[derive(Debug, Clone)]
pub struct NoiseConfig {
    pub layers: Vec<NoiseLayer>,
    pub value_range: (f32, f32),
}

impl NoiseConfig {
    pub fn new(base_scale: f32) -> Self {
        Self {
            layers: vec![NoiseLayer::new(base_scale, 1.0, 4)],
            value_range: (0.0, 1.0),
        }
    }

    pub fn with_layer(mut self, layer: NoiseLayer) -> Self {
        self.layers.push(layer);
        self
    }

    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.value_range = (min, max);
        self
    }
}

/// Complete terrain generation configuration
#[derive(Resource, Debug, Clone)]
pub struct TerrainGeneratorConfig {
    pub height_noise: NoiseConfig,
    pub moisture_noise: NoiseConfig,
    pub river_noise: NoiseConfig,
    pub detail_noise: NoiseConfig,
}

impl Default for TerrainGeneratorConfig {
    fn default() -> Self {
        // Height noise configuration
        let height_noise = NoiseConfig::new(100.0)
            .with_layer(
                NoiseLayer::new(50.0, 0.5, 2)
                    .with_offset(Vec2::new(1000.0, 1000.0))
                    .with_weight(0.5),
            )
            .with_layer(
                NoiseLayer::new(25.0, 0.25, 1)
                    .with_offset(Vec2::new(2000.0, 2000.0))
                    .with_weight(0.25),
            );

        // Moisture noise configuration
        let moisture_noise =
            NoiseConfig::new(150.0).with_layer(NoiseLayer::new(75.0, 0.5, 2).with_weight(0.3));

        // River noise configuration
        let river_noise = NoiseConfig::new(200.0)
            .with_layer(
                NoiseLayer::new(100.0, 1.0, 1)
                    .with_persistence(1.0)
                    .with_lacunarity(1.0),
            )
            .with_range(0.0, 0.8); // Rivers are more likely in lower values

        // Detail noise configuration
        let detail_noise = NoiseConfig::new(75.0).with_layer(
            NoiseLayer::new(25.0, 0.5, 1)
                .with_offset(Vec2::new(3000.0, 3000.0))
                .with_weight(0.3),
        );

        Self {
            height_noise,
            moisture_noise,
            river_noise,
            detail_noise,
        }
    }
}

#[derive(Resource)]
pub struct TerrainGeneratorSystem {
    height_generator: NoiseGenerator,
    moisture_generator: NoiseGenerator,
    river_generator: NoiseGenerator,
    detail_generator: NoiseGenerator,
    rng: StdRng,
}

impl TerrainGeneratorSystem {
    pub fn new(seed: u64, config: &TerrainGeneratorConfig) -> Self {
        let mut height_generator = NoiseGenerator::new(seed);
        let mut moisture_generator = NoiseGenerator::new(seed.wrapping_add(1));
        let mut river_generator = NoiseGenerator::new(seed.wrapping_add(2));
        let mut detail_generator = NoiseGenerator::new(seed.wrapping_add(3));

        // Configure generators based on config
        for layer in &config.height_noise.layers {
            height_generator.add_layer(layer.clone());
        }
        height_generator.set_value_range(
            config.height_noise.value_range.0,
            config.height_noise.value_range.1,
        );

        for layer in &config.moisture_noise.layers {
            moisture_generator.add_layer(layer.clone());
        }
        moisture_generator.set_value_range(
            config.moisture_noise.value_range.0,
            config.moisture_noise.value_range.1,
        );

        for layer in &config.river_noise.layers {
            river_generator.add_layer(layer.clone());
        }
        river_generator.set_value_range(
            config.river_noise.value_range.0,
            config.river_noise.value_range.1,
        );

        for layer in &config.detail_noise.layers {
            detail_generator.add_layer(layer.clone());
        }
        detail_generator.set_value_range(
            config.detail_noise.value_range.0,
            config.detail_noise.value_range.1,
        );

        Self {
            height_generator,
            moisture_generator,
            river_generator,
            detail_generator,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Generate all noise values for a given position
    pub fn generate_noise_values(&self, world_pos: Vec2) -> TerrainNoiseValues {
        TerrainNoiseValues {
            height: self.height_generator.get_noise(world_pos),
            moisture: self.moisture_generator.get_noise(world_pos),
            river: self.river_generator.get_noise(world_pos),
            detail: self.detail_generator.get_noise(world_pos),
        }
    }
}

/// Structure to hold all noise values for a point
#[derive(Debug, Clone, Copy)]
pub struct TerrainNoiseValues {
    pub height: f32,
    pub moisture: f32,
    pub river: f32,
    pub detail: f32,
}

impl FromWorld for TerrainGeneratorSystem {
    fn from_world(world: &mut World) -> Self {
        let seed = world.resource::<TerrainState>().seed;
        let config = world.resource::<TerrainGeneratorConfig>();
        Self::new(seed, config)
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
    let chunk_world_x = chunk.position.x * state.chunk_size as i32;
    let chunk_world_y = chunk.position.y * state.chunk_size as i32;
    let world_scale = config.noise_scale;

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let idx = (y * chunk_size + x) as usize;
            let world_x = if x == 0 && chunk.position.x > 0 {
                (chunk_world_x - 1) as f32
            } else {
                (chunk_world_x + x as i32) as f32
            };

            let world_y = if y == 0 && chunk.position.y > 0 {
                (chunk_world_y - 1) as f32
            } else {
                (chunk_world_y + y as i32) as f32
            };

            let world_pos = Vec2::new(world_x, world_y);

            chunk.height_map[idx] = generator.get_height(world_pos);
            chunk.moisture_map[idx] = generator.get_height(world_pos);

            let river_value = generator.get_river_value(world_pos);
            let detail_value = generator.get_detail_value(world_pos);

            let water_distance = distance_to_water(chunk, x, y, chunk_size);

            // Determine biome using noise values
            chunk.biome_map[idx] = generate_biome(
                chunk.height_map[idx],
                chunk.moisture_map[idx],
                river_value,
                detail_value,
                detail_value, // Using same value for micro/macro noise
                water_distance,
                config,
            );
        }
    }
    generate_rivers(chunk, state, generator, &config.river_config);

    smooth_biome_transitions(chunk, chunk_size);
}

struct BiomeData {
    biome: BiomeType,
    weight: f32,
}

fn generate_biome(
    height: f32,
    moisture: f32,
    river_noise: f32,
    macro_noise: f32,
    micro_noise: f32,
    water_distance: f32,
    config: &TerrainConfig,
) -> BiomeType {
    if river_noise < 0.2 && height < 0.6 {
        return BiomeType::Water;
    }

    // if height < config.water_threshold + config.beach_width || water_distance < 2.0 {
    //     return BiomeType::Sand;
    // }

    let mut biome_weights = Vec::new();
    let forest_weight = (moisture * 1.2).min(1.0) * (macro_noise * 0.8);
    let grass_weight = (1.0 - moisture) * (1.0 - macro_noise) * 1.2;
    let orchard_weight = moisture * micro_noise * 0.4;
    let crops_weight = (1.0 - moisture) * micro_noise * 0.6;

    biome_weights.push(BiomeData {
        biome: BiomeType::Forest,
        weight: forest_weight,
    });
    biome_weights.push(BiomeData {
        biome: BiomeType::Grass,
        weight: grass_weight,
    });
    biome_weights.push(BiomeData {
        biome: BiomeType::Orchard,
        weight: orchard_weight,
    });
    biome_weights.push(BiomeData {
        biome: BiomeType::Crops,
        weight: crops_weight,
    });

    biome_weights.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    biome_weights[0].biome
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
    let world_pos = Vec2::new(
        chunk.position.x as f32 * state.chunk_size as f32 * state.scale,
        chunk.position.y as f32 * state.chunk_size as f32 * state.scale,
    );

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let idx = y * chunk_size + x;
            let pos = world_pos + Vec2::new(x as f32, y as f32) * state.scale;
            let biome = chunk.biome_map[idx];

            // Try to spawn features based on biome
            if let Some(feature) = try_spawn_feature(
                pos,
                biome,
                chunk.height_map[idx],
                chunk.moisture_map[idx],
                config,
                rng,
            ) {
                spawn_feature(commands, chunk_entity, feature, assets);
            }
        }
    }
}

fn try_spawn_feature(
    pos: Vec2,
    biome: BiomeType,
    height: f32,
    moisture: f32,
    config: &TerrainConfig,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    match biome {
        BiomeType::Forest => try_spawn_tree(pos, height, moisture, config, rng),
        BiomeType::Orchard => try_spawn_orchard_tree(pos, height, moisture, config, rng),
        BiomeType::Crops => try_spawn_bush(pos, height, moisture, config, rng),
        BiomeType::Grass => try_spawn_flower(pos, height, moisture, config, rng),
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
    _height: f32,
    _moisture: f32,
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
    let density = config.get_feature_density(feature_type, BiomeType::Forest);

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
    _height: f32,
    _moisture: f32,
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
    if object_probability > config.get_feature_density(feature_type, BiomeType::Forest) {
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
    _height: f32,
    _moisture: f32,
    _config: &TerrainConfig,
    rng: &mut StdRng,
) -> Option<TerrainFeatureComponent> {
    // Using original crops logic for bush placement
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
    _height: f32,
    _moisture: f32,
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
    let density = config.get_feature_density(feature_type, BiomeType::Grass);

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
