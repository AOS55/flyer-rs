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

/// System responsible for generating procedural terrain.
/// It uses noise generators to generate height maps, moisture maps,
/// biome maps, and features like trees, crops, and rivers.
#[derive(Resource, Clone)]
pub struct TerrainGeneratorSystem {
    /// Noise generator for terrain height.
    height_generator: NoiseGenerator,
    /// Noise generator for terrain moisture.
    moisture_generator: NoiseGenerator,
    /// Noise generator for river generation.
    river_generator: NoiseGenerator,
    /// Noise generator for small terrain details.
    detail_generator: NoiseGenerator,
    /// Random number generator for consistent procedural generation.
    rng: StdRng,
}

impl TerrainGeneratorSystem {
    /// Creates a new terrain generator based on the provided `TerrainConfig`.
    pub fn new(config: &TerrainConfig) -> Self {
        let mut height_generator = NoiseGenerator::new(config.seed);
        let mut moisture_generator = NoiseGenerator::new(config.seed.wrapping_add(1));
        let mut river_generator = NoiseGenerator::new(config.seed.wrapping_add(2));
        let detail_generator = NoiseGenerator::new(config.seed.wrapping_add(3));

        // Configure height noise generator with layers
        height_generator.set_value_range(0.0, 1.0);
        for layer in &config.noise.height.layers {
            height_generator.add_layer(layer.clone());
        }

        // Configure moisture noise generator with layers
        moisture_generator.set_value_range(0.0, 1.0);
        for layer in &config.noise.moisture.layers {
            moisture_generator.add_layer(layer.clone());
        }

        // Configure river noise generator
        river_generator.set_value_range(0.0, config.noise.river.max_length);

        Self {
            height_generator,
            moisture_generator,
            river_generator,
            detail_generator,
            rng: StdRng::seed_from_u64(config.seed),
        }
    }

    /// Generates a single terrain chunk at the given position.
    ///
    /// # Arguments
    /// * `position` - Chunk position in chunk coordinates.
    /// * `state` - Reference to the terrain state.
    /// * `config` - Reference to the terrain configuration.
    ///
    /// # Returns
    /// A fully generated `TerrainChunkComponent` containing height, moisture, biomes, and features.
    pub fn generate_chunk(
        &mut self,
        position: IVec2,
        state: &TerrainState,
        config: &TerrainConfig,
    ) -> TerrainChunkComponent {
        // Initialize result structures
        let mut result = TerrainChunkComponent::new(position, state);

        // Generate height and moisture maps
        self.generate_base_terrain(state, &mut result);

        // Determine biomes based on height and moisture
        self.generate_biomes(state, &mut result, config);

        // Add features like trees, crops, etc.
        self.generate_features(state, &mut result, config);

        result
    }

    /// Generates the base terrain by populating the height and moisture maps.
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

    /// Generates the biome map based on height and moisture thresholds.
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
                    config.seed,
                    world_pos,
                );
            }
        }
        // Smooth biome transitions to reduce harsh boundaries.
        self.smooth_biome_transitions(&mut result.biome_map, state.chunk_size as i32);
    }

    /// Generates terrain features (e.g., trees, crops) based on biome and configuration.
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

    /// Fetches height noise value at a given position.
    pub fn get_height(&self, pos: Vec2) -> f32 {
        self.height_generator.get_noise(pos)
    }

    /// Fetches moisture noise value at a given position.
    pub fn get_moisture(&self, pos: Vec2) -> f32 {
        self.moisture_generator.get_noise(pos)
    }

    /// Fetches river noise value at a given position.
    pub fn get_river_value(&self, pos: Vec2) -> f32 {
        self.river_generator.get_noise(pos)
    }

    /// Fetches detail noise value at a given position.
    pub fn get_detail_value(&self, pos: Vec2) -> f32 {
        self.detail_generator.get_noise(pos)
    }

    /// Smoothes biome transitions using a weighted kernel to reduce abrupt biome changes.
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

/// Determines the biome for a tile based on height, moisture, and thresholds.
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

/// Smoothstep function to create smooth transitions between thresholds.
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

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
