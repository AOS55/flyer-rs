use bevy::prelude::*;

use crate::components::terrain::{BiomeType, TerrainChunkComponent};
use crate::resources::terrain::{config::RiverNoiseConfig, TerrainState};
use crate::systems::terrain::{generator::TerrainGeneratorSystem, noise::NoiseGenerator};

/// Represents a river segment with its properties
#[derive(Debug, Clone)]
struct RiverSegment {
    pos: Vec2,
    width: f32,
}

/// Represents a complete river with all its segments
#[derive(Debug)]
struct River {
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
            }],
            source_height,
            total_length: 0.0,
        }
    }

    fn add_segment(&mut self, pos: Vec2, current_height: f32, source_distance: f32) {
        if let Some(last) = self.segments.last() {
            self.total_length += (pos - last.pos).length();
        }

        // Calculate flow intensity based on height difference from source
        let height_diff = (self.source_height - current_height).max(0.0);
        let flow_intensity = (height_diff * 2.0).min(1.0);

        // Adjust width and depth based on flow intensity and distance
        let base_width = 1.0 + (source_distance / 100.0).min(3.0);
        let width = base_width * (0.5 + flow_intensity * 0.5);

        self.segments.push(RiverSegment { pos, width });
    }
}

pub fn generate_rivers(
    chunk: &mut TerrainChunkComponent,
    state: &TerrainState,
    generator: &TerrainGeneratorSystem,
    config: &RiverNoiseConfig,
) {
    let chunk_size = state.chunk_size as i32;
    let mut rivers = Vec::new();
    let mut sources = find_river_sources(chunk, chunk_size, config);

    // Sort sources by height to start with highest points
    sources.sort_by(|a, b| {
        let idx_a = (a.y as i32 * chunk_size + a.x as i32) as usize;
        let idx_b = (b.y as i32 * chunk_size + b.x as i32) as usize;
        chunk.height_map[idx_b]
            .partial_cmp(&chunk.height_map[idx_a])
            .unwrap()
    });

    for source in sources {
        if let Some(river) = generate_river_path(chunk, source, chunk_size, generator, config) {
            rivers.push(river);
        }
    }

    apply_rivers_to_terrain(chunk, &rivers, chunk_size, config);
}

fn find_river_sources(
    chunk: &TerrainChunkComponent,
    chunk_size: i32,
    config: &RiverNoiseConfig,
) -> Vec<Vec2> {
    let mut sources = Vec::new();
    let source_noise = NoiseGenerator::new(12345);

    for y in 0..chunk_size {
        for x in 0..chunk_size {
            let idx = (y * chunk_size + x) as usize;
            let height = chunk.height_map[idx];
            let pos = Vec2::new(x as f32, y as f32);

            if height > config.min_source_height {
                let noise_val = source_noise.get_noise(pos * 0.1);
                if noise_val > 0.7 {
                    sources.push(pos);
                }
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
    config: &RiverNoiseConfig,
) -> Option<River> {
    let source_idx = (source.y as i32 * chunk_size + source.x as i32) as usize;
    let source_height = chunk.height_map[source_idx];

    let mut river = River::new(source, source_height);
    let mut current_pos = source;
    let meander_noise = NoiseGenerator::new(54321);

    while river.total_length < config.max_length {
        let next_pos =
            find_next_river_point(chunk, current_pos, chunk_size, &meander_noise, config);

        if next_pos == current_pos {
            break;
        }

        let current_idx = (next_pos.y as i32 * chunk_size + next_pos.x as i32) as usize;
        let current_height = chunk.height_map[current_idx];

        river.add_segment(next_pos, current_height, river.total_length);
        current_pos = next_pos;

        // Check if we've reached water or the chunk boundary
        if current_idx >= chunk.biome_map.len() || chunk.biome_map[current_idx] == BiomeType::Water
        {
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
    config: &RiverNoiseConfig,
) -> Vec2 {
    let mut lowest_pos = current;
    let mut lowest_height = f32::MAX;
    let current_idx = (current.y as i32 * chunk_size + current.x as i32) as usize;
    let current_height = chunk.height_map[current_idx];

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = current.x + dx as f32;
            let ny = current.y + dy as f32;

            if nx < 0.0 || nx >= chunk_size as f32 || ny < 0.0 || ny >= chunk_size as f32 {
                continue;
            }

            let neighbor_idx = (ny as i32 * chunk_size + nx as i32) as usize;
            let height = chunk.height_map[neighbor_idx];

            let meander_value = meander_noise.get_noise(Vec2::new(nx, ny)) * config.meander_factor;
            let adjusted_height = height + meander_value;

            if adjusted_height < lowest_height && adjusted_height < current_height {
                lowest_height = adjusted_height;
                lowest_pos = Vec2::new(nx, ny);
            }
        }
    }

    lowest_pos
}

fn apply_rivers_to_terrain(
    chunk: &mut TerrainChunkComponent,
    rivers: &[River],
    chunk_size: i32,
    config: &RiverNoiseConfig,
) {
    for river in rivers {
        for segment in &river.segments {
            let radius = (segment.width * 1.5).ceil() as i32;

            // Calculate erosion strength based on height difference
            let height_diff = river.source_height
                - chunk.height_map
                    [(segment.pos.y as i32 * chunk_size + segment.pos.x as i32) as usize];
            let flow_intensity = (height_diff * 2.0).min(1.0);
            let erosion_modifier = 0.5 + flow_intensity * 0.5;

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let x = segment.pos.x as i32 + dx;
                    let y = segment.pos.y as i32 + dy;

                    if x < 0 || x >= chunk_size || y < 0 || y >= chunk_size {
                        continue;
                    }

                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    if distance > segment.width {
                        continue;
                    }

                    let idx = (y * chunk_size + x) as usize;

                    // Enhanced erosion based on flow intensity
                    let bank_factor = distance / segment.width;
                    let erosion = (1.0 - bank_factor) * config.erosion_strength * erosion_modifier;
                    chunk.height_map[idx] *= 1.0 - erosion;

                    // Deeper valley in steeper areas
                    let valley_width = segment.width * (2.0 + flow_intensity);
                    let valley_factor = (-((distance / valley_width).powi(2))).exp();
                    chunk.height_map[idx] -= valley_factor * 0.05 * (1.0 + flow_intensity);

                    // Update biome and moisture
                    if distance < segment.width * 0.7 {
                        chunk.biome_map[idx] = BiomeType::Water;
                    } else if distance < segment.width {
                        chunk.biome_map[idx] = BiomeType::Beach;
                    }

                    // Increased moisture near stronger flows
                    let moisture_factor = (-((distance / (segment.width * 3.0)).powi(2))).exp();
                    chunk.moisture_map[idx] = (chunk.moisture_map[idx]
                        + moisture_factor * (1.0 + flow_intensity * 0.5))
                        .min(1.0);
                }
            }
        }
    }
}
