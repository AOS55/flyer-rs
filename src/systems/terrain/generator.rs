use bevy::prelude::*;
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;
use noise::{NoiseFn, OpenSimplex};
use rand::prelude::*;
use std::collections::HashMap;

use crate::components::terrain::*;

#[derive(Resource)]
pub struct TerrainGeneratorSystem {
    noise_gen: OpenSimplex,
    rng: StdRng,
    field_density: f32,
    kd_tree: Option<KdTree<f32, 2>>,
}

impl TerrainGeneratorSystem {
    pub fn new(seed: u64) -> Self {
        Self {
            noise_gen: OpenSimplex::new(seed as u32),
            rng: StdRng::seed_from_u64(seed),
            field_density: 0.001,
            kd_tree: None,
        }
    }

    fn init_kd_tree(&mut self, chunk_size: u32) {
        let mut tree = KdTree::new();

        // Calculate number of biome points based on chunk size and density
        let n_points = ((chunk_size * chunk_size) as f32 * self.field_density) as usize;

        // Generate random biome points
        for _ in 0..n_points {
            let x = self.rng.gen::<f32>() * chunk_size as f32;
            let y = self.rng.gen::<f32>() * chunk_size as f32;
            let biome = select_random_biome(&mut self.rng) as usize;

            tree.add(&[x, y], biome);
        }

        self.kd_tree = Some(tree);
    }

    fn get_biome_from_position(&self, pos: Vec2) -> BiomeType {
        if let Some(tree) = &self.kd_tree {
            let (_distance, biome_idx) = tree.nearest_one(&[pos.x, pos.y], &squared_euclidean);
            match biome_idx {
                0 => BiomeType::Grass,
                1 => BiomeType::Forest,
                2 => BiomeType::Crops,
                3 => BiomeType::Orchard,
                _ => BiomeType::Grass, // Default fallback
            }
        } else {
            BiomeType::Grass // Fallback if no tree initialized
        }
    }
}

impl FromWorld for TerrainGeneratorSystem {
    fn from_world(world: &mut World) -> Self {
        let seed = world.resource::<TerrainState>().seed;
        Self::new(seed)
    }
}

/// System to generate terrain data for new chunks
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

    generator.init_kd_tree(terrain_state.chunk_size);

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

    // Calculate base world coordinates for this chunk
    let chunk_world_x = chunk.position.x * state.chunk_size as i32;
    let chunk_world_y = chunk.position.y * state.chunk_size as i32;

    // Pre-calculate scale for consistency
    let world_scale = config.noise_scale;

    // Generate height and moisture maps
    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let idx = (y * chunk_size + x) as usize;

            // Special handling for chunk boundaries in both x and y directions
            let world_x = if x == 0 && chunk.position.x > 0 {
                // Left edge (except for leftmost chunks)
                (chunk_world_x - 1) as f32
            } else {
                (chunk_world_x + x as i32) as f32
            };

            let world_y = if y == 0 && chunk.position.y > 0 {
                // Bottom edge (except for bottom chunks)
                (chunk_world_y - 1) as f32
            } else {
                (chunk_world_y + y as i32) as f32
            };

            // Calculate world coordinates
            let world_pos = Vec2::new(world_x, world_y);

            chunk.height_map[idx] = generate_noise_value(
                &generator.noise_gen,
                world_pos,
                world_scale,
                config.noise_octaves,
                config.noise_persistence,
                config.noise_lacunarity,
            );

            chunk.moisture_map[idx] = generate_noise_value(
                &generator.noise_gen,
                world_pos * config.moisture_scale,
                world_scale,
                config.noise_octaves - 1,
                config.noise_persistence,
                config.noise_lacunarity,
            );

            chunk.biome_map[idx] = determine_biome(
                chunk.height_map[idx],
                chunk.moisture_map[idx],
                world_pos,
                config,
                generator,
            );
        }
    }
}

fn generate_noise_value(
    noise: &OpenSimplex,
    pos: Vec2,
    scale: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
) -> f32 {
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut noise_value = 0.0;
    let mut weight = 0.0;

    // Use doubles for better precision
    let base_x = pos.x as f64;
    let base_y = pos.y as f64;
    let scale_d = scale as f64;

    for _ in 0..octaves {
        let sample_x = (base_x * frequency) / scale_d;
        let sample_y = (base_y * frequency) / scale_d;

        let noise_val = noise.get([sample_x as f64, sample_y as f64]) as f32;

        noise_value += noise_val * amplitude;
        weight += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity as f64;
    }

    (noise_value / weight + 1.0) * 0.5
}

fn determine_biome(
    height: f32,
    moisture: f32,
    pos: Vec2,
    config: &TerrainConfig,
    generator: &mut TerrainGeneratorSystem,
) -> BiomeType {
    // Check for water and beach first
    if height < config.water_threshold {
        return BiomeType::Water;
    }
    if height < config.water_threshold + config.beach_width {
        return BiomeType::Sand;
    }

    // Get base biome from KD-tree
    let base_biome = generator.get_biome_from_position(pos);

    // Apply moisture and height modifications
    let base_probability = config.get_biome_probability(height, moisture, base_biome);

    // Calculate probabilities for all biomes
    let mut biome_scores: HashMap<BiomeType, f32> = HashMap::new();
    biome_scores.insert(base_biome, base_probability);

    for (&biome, &weight) in &config.biome_weights {
        if biome != base_biome {
            let prob = config.get_biome_probability(height, moisture, biome);
            biome_scores.insert(biome, prob * weight * 0.5);
        }
    }

    // Select highest scoring biome
    biome_scores
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(biome, _)| biome)
        .unwrap_or(base_biome)
}

fn select_random_biome(rng: &mut StdRng) -> BiomeType {
    let roll = rng.gen_range(0..100);
    match roll {
        0..=30 => BiomeType::Grass,
        31..=50 => BiomeType::Forest,
        51..=65 => BiomeType::Crops,
        66..=80 => BiomeType::Orchard,
        81..=90 => BiomeType::Water,
        _ => BiomeType::Sand,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq; // Add this to Cargo.toml for float comparisons

    mod generator_initialization {
        use super::*;

        #[test]
        fn test_generator_initialization_with_seed() {
            let seed = 12345;
            let generator = TerrainGeneratorSystem::new(seed);

            assert!(generator.kd_tree.is_none());
            assert_eq!(generator.field_density, 0.001);
        }

        #[test]
        fn test_kd_tree_initialization() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let chunk_size = 32;

            generator.init_kd_tree(chunk_size);

            assert!(generator.kd_tree.is_some());
            let tree = generator.kd_tree.as_ref().unwrap();

            // Expected number of points based on density
            let expected_points =
                ((chunk_size * chunk_size) as f32 * generator.field_density) as usize;
            assert_eq!(tree.size(), expected_points);
        }

        #[test]
        fn test_deterministic_initialization() {
            let seed = 12345;
            let gen1 = TerrainGeneratorSystem::new(seed);
            let gen2 = TerrainGeneratorSystem::new(seed);

            // Generate some noise values and compare
            let pos = Vec2::new(1.0, 1.0);
            let noise1 = gen1.noise_gen.get([pos.x as f64, pos.y as f64]);
            let noise2 = gen2.noise_gen.get([pos.x as f64, pos.y as f64]);

            assert_eq!(noise1, noise2);
        }
    }

    mod noise_generation {
        use super::*;

        #[test]
        fn test_noise_value_range() {
            let generator = TerrainGeneratorSystem::new(12345);
            let pos = Vec2::new(1.0, 1.0);
            let noise_value = generate_noise_value(
                &generator.noise_gen,
                pos,
                1.0, // scale
                4,   // octaves
                0.5, // persistence
                2.0, // lacunarity
            );

            assert!(noise_value >= -1.0 && noise_value <= 1.0);
        }

        #[test]
        fn test_noise_consistency() {
            let generator = TerrainGeneratorSystem::new(12345);
            let pos = Vec2::new(1.0, 1.0);

            let value1 = generate_noise_value(&generator.noise_gen, pos, 1.0, 4, 0.5, 2.0);

            let value2 = generate_noise_value(&generator.noise_gen, pos, 1.0, 4, 0.5, 2.0);

            assert_relative_eq!(value1, value2);
        }
    }

    mod biome_determination {
        use super::*;

        fn setup_test_config() -> TerrainConfig {
            TerrainConfig {
                water_threshold: 0.3,
                beach_width: 0.05,
                noise_scale: 1.0,
                noise_octaves: 4,
                noise_persistence: 0.5,
                noise_lacunarity: 2.0,
                moisture_scale: 1.0,
                biome_weights: HashMap::from([
                    (BiomeType::Grass, 1.0),
                    (BiomeType::Forest, 1.0),
                    (BiomeType::Water, 1.0),
                    (BiomeType::Sand, 1.0),
                ]),
                ..Default::default()
            }
        }

        #[test]
        fn test_water_biome_threshold() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let config = setup_test_config();
            let pos = Vec2::new(1.0, 1.0);

            let biome = determine_biome(
                0.2, // height below water threshold
                0.5, // moisture
                pos,
                &config,
                &mut generator,
            );

            assert_eq!(biome, BiomeType::Water);
        }

        #[test]
        fn test_beach_biome_threshold() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let config = setup_test_config();
            let pos = Vec2::new(1.0, 1.0);

            let height = config.water_threshold + (config.beach_width / 2.0);
            let biome = determine_biome(height, 0.5, pos, &config, &mut generator);

            assert_eq!(biome, BiomeType::Sand);
        }
    }

    mod feature_spawning {
        use super::*;

        #[test]
        fn test_tree_spawn_properties() {
            let mut rng = StdRng::seed_from_u64(12345);
            let config = TerrainConfig::default();
            let pos = Vec2::new(1.0, 1.0);

            if let Some(feature) = try_spawn_tree(
                pos, 0.5, // height
                0.5, // moisture
                &config, &mut rng,
            ) {
                assert!(matches!(feature.feature_type, FeatureType::Tree(_)));
                assert_eq!(feature.position, pos);
                assert!(feature.scale >= 0.8 && feature.scale <= 1.2);
                assert!(feature.rotation >= 0.0 && feature.rotation <= std::f32::consts::TAU);
            }
        }

        #[test]
        fn test_feature_density_distribution() {
            let mut rng = StdRng::seed_from_u64(12345);
            let config = TerrainConfig::default();
            let pos = Vec2::new(1.0, 1.0);

            let mut feature_count = 0;
            let iterations = 1000;

            for _ in 0..iterations {
                if try_spawn_tree(pos, 0.5, 0.5, &config, &mut rng).is_some() {
                    feature_count += 1;
                }
            }

            // Check if feature density is within reasonable bounds
            let density = feature_count as f32 / iterations as f32;
            assert!(density > 0.0 && density < 1.0);
        }
    }
}

#[cfg(test)]
mod advanced_tests {
    use super::*;
    use std::time::Instant;

    mod chunk_generation {
        use super::*;

        fn setup_test_chunk(position: IVec2) -> TerrainChunkComponent {
            TerrainChunkComponent {
                position,
                height_map: vec![0.0; 32 * 32],
                moisture_map: vec![0.0; 32 * 32],
                biome_map: vec![BiomeType::Grass; 32 * 32],
            }
        }

        #[test]
        fn test_chunk_generation_completeness() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let mut chunk = setup_test_chunk(IVec2::new(0, 0));
            let state = TerrainState {
                chunk_size: 32,
                scale: 1.0,
                seed: 12345,
                ..Default::default()
            };
            let config = TerrainConfig::default();

            generate_chunk_data(&mut chunk, &state, &config, &mut generator);

            // Verify all maps are populated
            assert!(!chunk.height_map.iter().any(|&x| x == 0.0));
            assert!(!chunk.moisture_map.iter().any(|&x| x == 0.0));
            assert!(chunk.biome_map.iter().any(|&x| x != BiomeType::Grass));
        }

        #[test]
        fn test_chunk_border_consistency() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let state = TerrainState {
                chunk_size: 32,
                scale: 1.0,
                seed: 12345,
                ..Default::default()
            };
            let config = TerrainConfig::default();

            // Generate two adjacent chunks
            let mut chunk1 = setup_test_chunk(IVec2::new(0, 0));
            let mut chunk2 = setup_test_chunk(IVec2::new(1, 0));

            generate_chunk_data(&mut chunk1, &state, &config, &mut generator);
            generate_chunk_data(&mut chunk2, &state, &config, &mut generator);

            // Compare border values
            for y in 0..state.chunk_size as usize {
                let chunk1_border = chunk1.height_map
                    [y * state.chunk_size as usize + (state.chunk_size as usize - 1)];
                let chunk2_border = chunk2.height_map[y * state.chunk_size as usize];
                assert!((chunk1_border - chunk2_border).abs() < 0.1);
            }
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_extreme_height_values() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let config = TerrainConfig::default();
            let pos = Vec2::new(1.0, 1.0);

            // Test very high elevation
            let high_biome = determine_biome(1.0, 0.5, pos, &config, &mut generator);
            assert!(matches!(high_biome, BiomeType::Grass | BiomeType::Forest));

            // Test very low elevation
            let low_biome = determine_biome(0.0, 0.5, pos, &config, &mut generator);
            assert_eq!(low_biome, BiomeType::Water);
        }

        #[test]
        fn test_chunk_size_boundaries() {
            let mut generator = TerrainGeneratorSystem::new(12345);

            // Test minimum chunk size
            generator.init_kd_tree(4);
            assert!(generator.kd_tree.is_some());

            // Test large chunk size
            generator.init_kd_tree(256);
            assert!(generator.kd_tree.is_some());
        }
    }

    mod performance {
        use super::*;

        #[test]
        fn benchmark_chunk_generation() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let state = TerrainState {
                chunk_size: 32,
                scale: 1.0,
                seed: 12345,
                ..Default::default()
            };
            let config = TerrainConfig::default();
            let mut chunk = TerrainChunkComponent {
                position: IVec2::new(0, 0),
                height_map: vec![0.0; 32 * 32],
                moisture_map: vec![0.0; 32 * 32],
                biome_map: vec![BiomeType::Grass; 32 * 32],
            };

            let start = Instant::now();
            generate_chunk_data(&mut chunk, &state, &config, &mut generator);
            let duration = start.elapsed();

            // Assert generation takes less than 50ms (adjust based on your requirements)
            assert!(duration.as_millis() < 50);
        }

        #[test]
        fn test_kd_tree_performance() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let chunk_size = 32;

            let start = Instant::now();
            generator.init_kd_tree(chunk_size);
            let init_duration = start.elapsed();

            // Test point lookup performance
            let start = Instant::now();
            for x in 0..10 {
                for y in 0..10 {
                    let pos = Vec2::new(x as f32, y as f32);
                    generator.get_biome_from_position(pos);
                }
            }
            let lookup_duration = start.elapsed();

            // Assert reasonable performance bounds
            assert!(init_duration.as_millis() < 10);
            assert!(lookup_duration.as_micros() < 1000);
        }
    }

    mod biome_distribution {
        use super::*;

        #[test]
        fn test_biome_distribution_ratios() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let state = TerrainState {
                chunk_size: 32,
                scale: 1.0,
                seed: 12345,
                ..Default::default()
            };
            let config = TerrainConfig::default();
            let mut chunk = TerrainChunkComponent {
                position: IVec2::new(0, 0),
                height_map: vec![0.0; 32 * 32],
                moisture_map: vec![0.0; 32 * 32],
                biome_map: vec![BiomeType::Grass; 32 * 32],
            };

            generate_chunk_data(&mut chunk, &state, &config, &mut generator);

            let mut biome_counts = HashMap::new();
            for &biome in &chunk.biome_map {
                *biome_counts.entry(biome).or_insert(0) += 1;
            }

            // Ensure we have a mix of biomes
            assert!(biome_counts.len() >= 3);

            // Check that no single biome dominates completely
            let total_tiles = (state.chunk_size * state.chunk_size) as usize;
            for &count in biome_counts.values() {
                assert!(count < total_tiles * 3 / 4);
            }
        }

        #[test]
        fn test_biome_transitions() {
            let mut generator = TerrainGeneratorSystem::new(12345);
            let config = TerrainConfig::default();
            let pos = Vec2::new(1.0, 1.0);

            // Test moisture transition
            let dry_biome = determine_biome(0.5, 0.1, pos, &config, &mut generator);
            let wet_biome = determine_biome(0.5, 0.9, pos, &config, &mut generator);
            assert_ne!(dry_biome, wet_biome);

            // Test height transition
            let low_biome = determine_biome(0.3, 0.5, pos, &config, &mut generator);
            let high_biome = determine_biome(0.7, 0.5, pos, &config, &mut generator);
            assert_ne!(low_biome, high_biome);
        }
    }

    mod feature_variants {
        use super::*;

        #[test]
        fn test_tree_variant_distribution() {
            let mut rng = StdRng::seed_from_u64(12345);
            let config = TerrainConfig::default();
            let pos = Vec2::new(1.0, 1.0);

            let mut variants = HashMap::new();
            for _ in 0..1000 {
                if let Some(feature) = try_spawn_tree(pos, 0.5, 0.5, &config, &mut rng) {
                    if let FeatureType::Tree(variant) = feature.feature_type {
                        *variants.entry(variant).or_insert(0) += 1;
                    }
                }
            }

            // Check that we have multiple variants
            assert!(variants.len() >= 2);

            // Check distribution isn't heavily skewed
            for &count in variants.values() {
                assert!(count > 50); // At least 5% of spawns
            }
        }

        #[test]
        fn test_feature_scale_distribution() {
            let mut rng = StdRng::seed_from_u64(12345);
            let config = TerrainConfig::default();
            let pos = Vec2::new(1.0, 1.0);

            let mut scales = Vec::new();
            for _ in 0..100 {
                if let Some(feature) = try_spawn_tree(pos, 0.5, 0.5, &config, &mut rng) {
                    scales.push(feature.scale);
                }
            }

            // Check scale range
            let min_scale = scales.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_scale = scales.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

            assert!(min_scale >= 0.8);
            assert!(max_scale <= 1.2);
            assert!((max_scale - min_scale) > 0.2); // Ensure good variation
        }
    }
}
