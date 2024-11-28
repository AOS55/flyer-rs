use crate::resources::config::TerrainConfig;
use crate::utils::errors::SimError;
use kiddo::{distance::squared_euclidean, KdTree};
use nalgebra::Vector3;
use noise::{NoiseFn, OpenSimplex};

pub struct TerrainGenerator {
    noise: OpenSimplex,
    config: TerrainConfig,
    kdtree: KdTree<f32, 2>,
    biome_cache: Vec<BiomeData>,
}

struct BiomeData {
    position: Vector3<f64>,
    biome_type: BiomeType,
    height_offset: f64,
}

#[derive(Clone, Copy)]
pub enum BiomeType {
    Plains,
    Mountains,
    Forest,
    Water,
}

impl TerrainGenerator {
    pub fn new(config: TerrainConfig) -> Self {
        let noise = OpenSimplex::new(config.seed);
        let kdtree = Self::initialize_biomes(&config);

        Self {
            noise,
            config,
            kdtree,
            biome_cache: Vec::new(),
        }
    }

    pub fn generate_chunk(
        &mut self,
        position: Vector3<f64>,
    ) -> Result<Option<TerrainChunk>, SimError> {
        let chunk_size = self.config.chunk_size;
        let mut chunk = TerrainChunk::new(position, chunk_size);

        for x in 0..chunk_size {
            for y in 0..chunk_size {
                let world_x = position.x + x as f64 * self.config.scale;
                let world_y = position.y + y as f64 * self.config.scale;

                let height = self.generate_height(world_x, world_y)?;
                let biome = self.get_biome_at(world_x, world_y);

                chunk.set_height(x, y, height);
                chunk.set_biome(x, y, biome);
            }
        }

        Ok(Some(chunk))
    }

    fn generate_height(&self, x: f64, y: f64) -> Result<f64, SimError> {
        let base = self
            .noise
            .get([x * self.config.frequency, y * self.config.frequency, 0.0]);

        let mountain = self.noise.get([
            x * self.config.frequency * 2.0,
            y * self.config.frequency * 2.0,
            100.0,
        ]);

        let detail = self.noise.get([
            x * self.config.frequency * 4.0,
            y * self.config.frequency * 4.0,
            200.0,
        ]);

        let height = base * 0.5 + mountain * 0.3 + detail * 0.2;
        Ok(height * self.config.amplitude)
    }

    fn initialize_biomes(config: &TerrainConfig) -> KdTree<f32, 2> {
        let mut tree = KdTree::new();
        let mut rng = rand::thread_rng();

        // Implementation follows your existing biome generation logic
        // but adapted for the KD-tree structure
        tree
    }

    fn get_biome_at(&self, x: f64, y: f64) -> BiomeType {
        let nearest = self
            .kdtree
            .nearest_one(&[x as f32, y as f32], &squared_euclidean);

        // Convert KD-tree result to BiomeType
        match nearest.1 {
            0 => BiomeType::Plains,
            1 => BiomeType::Mountains,
            2 => BiomeType::Forest,
            _ => BiomeType::Water,
        }
    }
}

pub struct TerrainChunk {
    position: Vector3<f64>,
    size: usize,
    heights: Vec<f64>,
    biomes: Vec<BiomeType>,
}

impl TerrainChunk {
    pub fn new(position: Vector3<f64>, size: usize) -> Self {
        Self {
            position,
            size,
            heights: vec![0.0; size * size],
            biomes: vec![BiomeType::Plains; size * size],
        }
    }

    pub fn set_height(&mut self, x: usize, y: usize, height: f64) {
        if x < self.size && y < self.size {
            self.heights[y * self.size + x] = height;
        }
    }

    pub fn set_biome(&mut self, x: usize, y: usize, biome: BiomeType) {
        if x < self.size && y < self.size {
            self.biomes[y * self.size + x] = biome;
        }
    }

    pub fn get_height(&self, x: usize, y: usize) -> Option<f64> {
        if x < self.size && y < self.size {
            Some(self.heights[y * self.size + x])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_generation() {
        let config = TerrainConfig {
            seed: 42,
            frequency: 0.01,
            amplitude: 100.0,
            chunk_size: 16,
            scale: 1.0,
        };

        let mut generator = TerrainGenerator::new(config);
        let position = Vector3::new(0.0, 0.0, 0.0);
        let chunk = generator.generate_chunk(position).unwrap().unwrap();

        assert!(chunk.get_height(0, 0).unwrap() >= -100.0);
        assert!(chunk.get_height(0, 0).unwrap() <= 100.0);
    }
}
