use crate::components::camera::CameraComponent;
use crate::ecs::component::Component;
use crate::utils::CHUNK_SIZE;

use glam::{UVec2, Vec2};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Serialize, Deserialize)]
pub struct TerrainTile {
    pub name: String,
    pub asset: String,
    pub biome_type: BiomeType,
    pub position: Vec2,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TerrainFeature {
    pub name: String,
    pub asset: String,
    pub feature_type: FeatureType,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: f32,
}

#[derive(Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum BiomeType {
    Grass,
    Forest,
    Crops,
    Orchard,
    Water,
    Sand,
}

#[derive(Clone, Copy, PartialEq, Hash, Serialize, Deserialize, Eq)]
pub enum FeatureType {
    Tree { variant: TreeVariant },
    Bush { variant: BushVariant },
    Flower { variant: FlowerVariant },
    Rock,
}

#[derive(Clone, Copy, PartialEq, Hash, Serialize, Deserialize, Eq)]
pub enum TreeVariant {
    EvergreenFir,
    WiltingFir,
    AppleTree,
    PrunedTree,
}

#[derive(Clone, Copy, PartialEq, Hash, Serialize, Deserialize, Eq)]
pub enum BushVariant {
    GreenBushel,
    RipeBushel,
    DeadBushel,
}

#[derive(Clone, Copy, PartialEq, Hash, Serialize, Deserialize, Eq)]
pub enum FlowerVariant {
    Single,
    Double,
    Quad,
    Cluster,
}

#[derive(Clone)]
pub struct TerrainChunk {
    pub biome_map: Vec<BiomeType>,
    pub height_map: Vec<f32>,
    pub moisture_map: Vec<f32>,
    pub tiles: Vec<TerrainTile>,
    pub features: Vec<TerrainFeature>,
    pub dirty: bool,
}

impl TerrainChunk {
    pub fn new(size: u32) -> Self {
        Self {
            biome_map: vec![BiomeType::Grass; (size * size) as usize],
            height_map: vec![0.0; (size * size) as usize],
            moisture_map: vec![0.0; (size * size) as usize],
            tiles: Vec::new(),
            features: Vec::new(),
            dirty: true,
        }
    }
}

#[derive(Clone)]
pub struct TerrainGenConfig {
    // Noise generation settings
    pub noise_scale: f32,       // Base scale for noise generation
    pub noise_octaves: u32,     // Number of octaves for noise generation
    pub noise_persistence: f32, // How much each octave contributes
    pub noise_lacunarity: f32,  // How frequency increases with each octave

    // Biome generation settings
    pub field_density: f32,  // Controls density of biome regions
    pub biome_scale: f32,    // Scale factor for biome transitions
    pub moisture_scale: f32, // Scale for moisture variation

    // Height thresholds
    pub water_threshold: f32, // Height below which water appears
    pub beach_width: f32,     // Width of beach transition

    // Feature placement settings
    pub feature_densities: HashMap<FeatureType, f32>,
    pub biome_weights: HashMap<BiomeType, f32>, // Relative weights for biome distribution
}

impl Default for TerrainGenConfig {
    fn default() -> Self {
        let mut feature_densities = HashMap::new();
        feature_densities.insert(
            FeatureType::Tree {
                variant: TreeVariant::EvergreenFir,
            },
            0.6,
        );
        feature_densities.insert(
            FeatureType::Tree {
                variant: TreeVariant::AppleTree,
            },
            0.1,
        );
        feature_densities.insert(
            FeatureType::Bush {
                variant: BushVariant::GreenBushel,
            },
            0.2,
        );
        feature_densities.insert(
            FeatureType::Flower {
                variant: FlowerVariant::Single,
            },
            0.1,
        );

        let mut biome_weights = HashMap::new();
        biome_weights.insert(BiomeType::Grass, 1.0);
        biome_weights.insert(BiomeType::Forest, 0.8);
        biome_weights.insert(BiomeType::Crops, 0.4);
        biome_weights.insert(BiomeType::Orchard, 0.3);
        biome_weights.insert(BiomeType::Water, 0.2);
        biome_weights.insert(BiomeType::Sand, 0.1);

        Self {
            // Noise settings
            noise_scale: 100.0,
            noise_octaves: 4,
            noise_persistence: 0.5,
            noise_lacunarity: 2.0,

            // Biome settings
            field_density: 0.005,
            biome_scale: 1.0,
            moisture_scale: 0.5,

            // Height thresholds
            water_threshold: -0.2,
            beach_width: 0.05,

            // Feature settings
            feature_densities,
            biome_weights,
        }
    }
}

impl TerrainGenConfig {
    // Helper method to get biome probability based on height and moisture
    pub fn get_biome_probability(&self, height: f32, moisture: f32, biome: BiomeType) -> f32 {
        let base_weight = self.biome_weights.get(&biome).unwrap_or(&0.0);

        match biome {
            BiomeType::Water => {
                if height < self.water_threshold {
                    1.0
                } else {
                    0.0
                }
            }
            BiomeType::Sand => {
                if height < self.water_threshold + self.beach_width {
                    1.0
                } else {
                    0.0
                }
            }
            BiomeType::Forest => {
                if moisture > 0.6 && height > self.water_threshold + self.beach_width {
                    base_weight * 1.5
                } else {
                    base_weight * 0.5
                }
            }
            BiomeType::Grass => {
                if height > self.water_threshold + self.beach_width {
                    base_weight * (1.0 - moisture)
                } else {
                    0.0
                }
            }
            _ => *base_weight,
        }
    }

    // Helper method to get feature density for a specific biome
    pub fn get_feature_density(&self, feature: FeatureType, biome: BiomeType) -> f32 {
        let base_density = self.feature_densities.get(&feature).unwrap_or(&0.0);

        match (feature, biome) {
            (FeatureType::Tree { .. }, BiomeType::Forest) => base_density * 1.5,
            (FeatureType::Tree { .. }, BiomeType::Orchard) => base_density * 1.2,
            (FeatureType::Bush { .. }, BiomeType::Crops) => base_density * 1.3,
            (FeatureType::Flower { .. }, BiomeType::Grass) => base_density * 1.1,
            _ => *base_density,
        }
    }
}

pub struct TerrainComponent {
    pub chunks: HashMap<UVec2, TerrainChunk>,
    pub chunk_size: u32,
    pub world_size: UVec2,
    pub scale: f32,
    pub seed: u64,
    pub config: TerrainGenConfig,
    pub active_chunks: Vec<UVec2>,
}

impl TerrainComponent {
    pub fn new(world_size: UVec2, chunk_size: u32, seed: u64, scale: f32) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size,
            world_size,
            scale,
            seed,
            config: TerrainGenConfig::default(),
            active_chunks: Vec::new(),
        }
    }

    pub fn get_chunk(&self, pos: UVec2) -> Option<&TerrainChunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: UVec2) -> Option<&mut TerrainChunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn world_to_chunk_pos(&self, world_pos: Vec2) -> UVec2 {
        UVec2::new(
            (world_pos.x / (self.chunk_size as f32 * self.scale)).floor() as u32,
            (world_pos.y / (self.chunk_size as f32 * self.scale)).floor() as u32,
        )
    }

    pub fn chunk_to_world_pos(&self, chunk_pos: UVec2) -> Vec2 {
        Vec2::new(
            chunk_pos.x as f32 * self.chunk_size as f32 * self.scale,
            chunk_pos.y as f32 * self.chunk_size as f32 * self.scale,
        )
    }

    pub fn get_biome_at(&self, world_pos: Vec2) -> Option<BiomeType> {
        let chunk_pos = self.world_to_chunk_pos(world_pos);
        let chunk = self.get_chunk(chunk_pos)?;

        let local_pos = world_pos - self.chunk_to_world_pos(chunk_pos);
        let x = (local_pos.x / self.scale) as usize;
        let y = (local_pos.y / self.scale) as usize;

        if x >= self.chunk_size as usize || y >= self.chunk_size as usize {
            return None;
        }

        Some(chunk.biome_map[y * self.chunk_size as usize + x])
    }

    pub fn get_visible_chunks(camera: &CameraComponent) -> HashSet<UVec2> {
        let mut visible = HashSet::new();

        // Calculate view distance based on viewport and zoom
        let viewport_extent = camera.viewport * 0.5 / camera.zoom;
        let view_distance = viewport_extent.length() / CHUNK_SIZE as f32;

        let chunk_pos = camera.position / CHUNK_SIZE as f32;
        let chunks_to_load = view_distance.ceil() as i32;

        // Add chunks in view distance
        for x in -chunks_to_load..=chunks_to_load {
            for y in -chunks_to_load..=chunks_to_load {
                visible.insert(UVec2::new(
                    (chunk_pos.x + x as f32) as u32,
                    (chunk_pos.y + y as f32) as u32,
                ));
            }
        }

        visible
    }
}

impl Component for TerrainComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
