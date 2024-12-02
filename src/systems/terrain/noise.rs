use crate::components::TerrainGenConfig;
use glam::Vec2;
use noise::{NoiseFn, OpenSimplex, Seedable};

pub struct NoiseGenerator {
    noise: OpenSimplex,
    seed: u64,
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            noise: OpenSimplex::new(seed as u32),
            seed,
        }
    }

    // Generate noise with multiple octaves for more natural terrain
    pub fn get_noise(
        &self,
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

        for _ in 0..octaves {
            let sample_x = pos.x as f64 * frequency as f64 / scale as f64;
            let sample_y = pos.y as f64 * frequency as f64 / scale as f64;

            let noise_val = self.noise.get([sample_x, sample_y]) as f32;
            noise_value += noise_val * amplitude;

            weight += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        // Normalize the result
        noise_value / weight
    }

    // Generate combined height and moisture maps for biome determination
    pub fn generate_terrain_maps(
        &self,
        chunk_pos: Vec2,
        chunk_size: u32,
        config: &TerrainGenConfig,
    ) -> (Vec<f32>, Vec<f32>) {
        let size = chunk_size as usize;
        let mut height_map = vec![0.0; size * size];
        let mut moisture_map = vec![0.0; size * size];

        for y in 0..size {
            for x in 0..size {
                let world_pos = Vec2::new(chunk_pos.x + x as f32, chunk_pos.y + y as f32);

                // Generate height with multiple octaves
                height_map[y * size + x] = self.get_noise(
                    world_pos,
                    config.noise_scale,
                    config.noise_octaves,
                    config.noise_persistence,
                    config.noise_lacunarity,
                );

                // Generate moisture with different parameters for variation
                moisture_map[y * size + x] = self.get_noise(
                    world_pos,
                    config.noise_scale * config.moisture_scale,
                    config.noise_octaves - 1,
                    config.noise_persistence,
                    config.noise_lacunarity,
                );
            }
        }

        (height_map, moisture_map)
    }

    // Generate variation noise for feature placement
    pub fn get_feature_variation(&self, world_pos: Vec2, scale: f32) -> f32 {
        self.get_noise(
            world_pos, scale, 2, // Fewer octaves for feature variation
            0.5, 2.0,
        )
    }

    // Generate noise for feature rotation
    pub fn get_feature_rotation(&self, world_pos: Vec2) -> f32 {
        self.get_noise(
            world_pos, 50.0, // Larger scale for smoother rotation variation
            1,    // Single octave is enough for rotation
            1.0, 1.0,
        ) * std::f32::consts::TAU // Full rotation range
    }
}
