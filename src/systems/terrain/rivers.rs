use bevy::prelude::*;

use crate::components::terrain::{BiomeType, RiverConfig, TerrainChunkComponent, TerrainState};
use crate::systems::terrain::{generator::TerrainGeneratorSystem, noise::NoiseGenerator};

/// Represents a river segment with its properties
#[derive(Debug, Clone)]
pub struct RiverSegment {
    pos: Vec2,
    width: f32,
    flow_strength: f32,
    depth: f32,
}

/// Represents a complete river with all its segments
#[derive(Debug)]
pub struct River {
    segments: Vec<RiverSegment>,
    source_height: f32,
    total_length: f32,
}

impl River {
    fn new(source_pos: Vec2, source_height: f32) -> Self {
        Self {
            segments: vec![RiverSegment {
                pos: source_pos,
                width: 1.0,
                flow_strength: 1.0,
                depth: 0.2,
            }],
            source_height,
            total_length: 0.0,
        }
    }

    fn add_segment(&mut self, segment: RiverSegment) {
        if let Some(last) = self.segments.last() {
            self.total_length += (segment.pos - last.pos).length();
        }
        self.segments.push(segment);
    }
}

pub fn generate_rivers(
    chunk: &mut TerrainChunkComponent,
    state: &TerrainState,
    generator: &TerrainGeneratorSystem,
    config: &RiverConfig,
) {
    let chunk_size = state.chunk_size as i32;
    let mut rivers = Vec::new();
    let mut river_sources = find_river_sources(chunk, chunk_size, config);

    // Sort sources by height to start with highest points
    river_sources.sort_by(|a, b| {
        let idx_a = (a.y as i32 * chunk_size + a.x as i32) as usize;
        let idx_b = (b.y as i32 * chunk_size + b.x as i32) as usize;
        chunk.height_map[idx_b]
            .partial_cmp(&chunk.height_map[idx_a])
            .unwrap()
    });

    // Generate each river
    for source in river_sources {
        if let Some(river) = generate_river_path(chunk, source, chunk_size, generator, config) {
            rivers.push(river);
        }
    }

    // Apply rivers to terrain
    apply_rivers_to_terrain(chunk, &rivers, chunk_size, config);
}

fn find_river_sources(
    chunk: &TerrainChunkComponent,
    chunk_size: i32,
    config: &RiverConfig,
) -> Vec<Vec2> {
    let mut sources = Vec::new();
    let source_noise = NoiseGenerator::new(12345); // Use consistent seed for source distribution

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let idx = (y * chunk_size + x) as usize;
            let height = chunk.height_map[idx];
            let pos = Vec2::new(x as f32, y as f32);

            // Use noise to add randomness to source placement
            let noise_val = source_noise.get_noise(pos * 0.1);

            if height > config.min_source_height && noise_val > 0.7 {
                sources.push(pos);
            }
        }
    }

    sources
}

fn generate_river_path(
    chunk: &TerrainChunkComponent,
    source: Vec2,
    chunk_size: i32,
    generator: &TerrainGeneratorSystem,
    config: &RiverConfig,
) -> Option<River> {
    let mut river = River::new(
        source,
        chunk.height_map[(source.y as i32 * chunk_size + source.x as i32) as usize],
    );
    let mut current_pos = source;
    let meander_noise = NoiseGenerator::new(54321); // Consistent seed for meandering

    while river.total_length < config.max_river_length {
        let (next_pos, flow_dir) =
            find_next_river_point(chunk, current_pos, chunk_size, &meander_noise, config);

        // Stop if we can't flow anywhere
        if next_pos == current_pos {
            break;
        }

        // Calculate new segment properties
        let segment_length = (next_pos - current_pos).length();
        let downstream_factor = river.total_length / config.max_river_length;
        let width = 1.0 + downstream_factor * config.width_growth_rate;
        let depth = 0.2 + downstream_factor * config.depth_growth_rate;
        let flow_strength = 1.0 - downstream_factor * 0.5;

        river.add_segment(RiverSegment {
            pos: next_pos,
            width,
            flow_strength,
            depth,
        });

        current_pos = next_pos;

        // Stop if we reach water or chunk boundary
        if !is_valid_river_position(chunk, current_pos, chunk_size) {
            break;
        }
    }

    if river.segments.len() > 1 {
        Some(river)
    } else {
        None
    }
}

fn find_next_river_point(
    chunk: &TerrainChunkComponent,
    current: Vec2,
    chunk_size: i32,
    meander_noise: &NoiseGenerator,
    config: &RiverConfig,
) -> (Vec2, Vec2) {
    let mut lowest_pos = current;
    let mut lowest_height = f32::MAX;
    let current_idx = (current.y as i32 * chunk_size + current.x as i32) as usize;
    let current_height = chunk.height_map[current_idx];

    // Check all neighboring positions
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = current.x + dx as f32;
            let ny = current.y + dy as f32;

            if !is_valid_river_position(chunk, Vec2::new(nx, ny), chunk_size) {
                continue;
            }

            let neighbor_idx = (ny as i32 * chunk_size + nx as i32) as usize;
            let height = chunk.height_map[neighbor_idx];

            // Add meandering effect using noise
            let meander_value = meander_noise.get_noise(Vec2::new(nx, ny)) * config.meander_factor;
            let adjusted_height = height + meander_value;

            if adjusted_height < lowest_height && adjusted_height < current_height {
                lowest_height = adjusted_height;
                lowest_pos = Vec2::new(nx, ny);
            }
        }
    }

    (lowest_pos, (lowest_pos - current).normalize())
}

fn apply_rivers_to_terrain(
    chunk: &mut TerrainChunkComponent,
    rivers: &[River],
    chunk_size: i32,
    config: &RiverConfig,
) {
    for river in rivers {
        for segment in &river.segments {
            apply_river_segment(chunk, segment, chunk_size, config);
        }
    }
}

fn apply_river_segment(
    chunk: &mut TerrainChunkComponent,
    segment: &RiverSegment,
    chunk_size: i32,
    config: &RiverConfig,
) {
    let radius = (segment.width / 2.0).ceil() as i32;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = segment.pos.x as i32 + dx;
            let y = segment.pos.y as i32 + dy;

            if !is_valid_position(x, y, chunk_size) {
                continue;
            }

            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            if distance > segment.width / 2.0 {
                continue;
            }

            let idx = (y * chunk_size + x) as usize;

            // Calculate erosion effect
            let erosion_factor = 1.0 - (distance / (segment.width / 2.0));
            let erosion = erosion_factor * config.erosion_strength * segment.flow_strength;

            // Apply river effects
            chunk.height_map[idx] *= 1.0 - erosion;
            chunk.biome_map[idx] = BiomeType::Water;

            // Could also modify moisture map here
            chunk.moisture_map[idx] = (chunk.moisture_map[idx] + erosion_factor).min(1.0);
        }
    }
}

fn is_valid_river_position(chunk: &TerrainChunkComponent, pos: Vec2, chunk_size: i32) -> bool {
    let x = pos.x as i32;
    let y = pos.y as i32;

    if !is_valid_position(x, y, chunk_size) {
        return false;
    }

    let idx = (y * chunk_size + x) as usize;
    chunk.biome_map[idx] != BiomeType::Water
}

fn is_valid_position(x: i32, y: i32, chunk_size: i32) -> bool {
    x >= 0 && x < chunk_size && y >= 0 && y < chunk_size
}
