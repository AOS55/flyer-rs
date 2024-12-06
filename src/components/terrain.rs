use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---- Components ----

#[derive(Component, Debug, Clone)]
pub struct TerrainFeatureComponent {
    pub feature_type: FeatureType,
    pub variant: FeatureVariant,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: f32,
}

#[derive(Component, Debug, Clone)]
#[require(Sprite, Transform)]
pub struct TerrainTileComponent {
    pub biome_type: BiomeType,
    pub position: Vec2,
    pub sprite_index: usize,
}

#[derive(Component, Debug, Clone)]
#[require(Sprite, Transform, Visibility)]
pub struct TerrainChunkComponent {
    pub position: IVec2,           // Chunk position in chunk coordinates
    pub height_map: Vec<f32>,      // Height values for each tile
    pub moisture_map: Vec<f32>,    // Moisture values for each tile
    pub biome_map: Vec<BiomeType>, // Biome type for each tile
}

impl TerrainChunkComponent {
    pub fn new(position: IVec2, chunk_size: u32) -> Self {
        let size = (chunk_size * chunk_size) as usize;
        Self {
            position,
            height_map: vec![0.0; size],
            moisture_map: vec![0.0; size],
            biome_map: vec![BiomeType::Grass; size],
        }
    }

    pub fn world_position(&self, chunk_size: u32, scale: f32) -> Vec3 {
        Vec3::new(
            self.position.x as f32 * chunk_size as f32 * scale,
            self.position.y as f32 * chunk_size as f32 * scale,
            0.0,
        )
    }
}

// ---- Resources ----

#[derive(Resource, Debug, Clone)]
pub struct TerrainState {
    pub chunk_size: u32,
    pub world_size: IVec2,
    pub scale: f32,
    pub seed: u64,
    pub active_chunks: Vec<IVec2>,
}

impl Default for TerrainState {
    fn default() -> Self {
        Self {
            chunk_size: 32,
            world_size: IVec2::new(1000, 1000),
            scale: 1.0,
            seed: 12345,
            active_chunks: Vec::new(),
        }
    }
}

#[derive(Resource, Clone, Debug, Deserialize, Serialize)]
pub struct TerrainConfig {
    // Noise generation settings
    pub noise_scale: f32,
    pub noise_octaves: u32,
    pub noise_persistence: f32,
    pub noise_lacunarity: f32,

    // Biome generation settings
    pub field_density: f32,
    pub biome_scale: f32,
    pub moisture_scale: f32,

    // Height thresholds
    pub water_threshold: f32,
    pub beach_width: f32,

    // Feature placement settings
    pub feature_densities: HashMap<FeatureType, f32>,
    pub biome_weights: HashMap<BiomeType, f32>,

    // River generation settings
    pub river_config: RiverConfig,
}

/// Resource that manages all terrain-related textures and their mappings
#[derive(Resource, Clone)]
pub struct TerrainAssets {
    /// Handle to the main terrain tileset image (e.g., "terrain_tiles.png")
    /// Contains all base terrain types (grass, water, sand, etc.) in a sprite sheet
    pub tile_texture: Handle<Image>,

    /// Handle to the feature tileset image (e.g., "features.png")
    /// Contains all terrain features (trees, rocks, buildings, etc.) in a sprite sheet
    pub feature_texture: Handle<Image>,

    /// Defines how to split the tile_texture into individual sprites
    /// Specifies the grid layout (tile size, rows, columns) for terrain tiles
    pub tile_layout: Handle<TextureAtlasLayout>,

    /// Defines how to split the feature_texture into individual sprites
    /// Specifies the grid layout (sprite size, rows, columns) for feature sprites
    pub feature_layout: Handle<TextureAtlasLayout>,

    /// Maps game biome types to their corresponding sprite indices in tile_texture
    /// Example: BiomeType::Desert => 0 (first sprite in the tileset)
    pub tile_mappings: HashMap<BiomeType, usize>,

    /// Maps game feature types to their corresponding sprite indices in feature_texture
    /// Example: FeatureType::Tree => 5 (sixth sprite in the feature set)
    pub feature_mappings: HashMap<FeatureType, usize>,
}

/// Configuration for river generation
#[derive(Debug, Clone, Resource, Deserialize, Serialize)]
pub struct RiverConfig {
    pub min_source_height: f32, // Minimum height for river sources
    pub meander_factor: f32,    // How much rivers can deviate from steepest path
    pub min_slope: f32,         // Minimum slope required for river flow
    pub width_growth_rate: f32, // How quickly rivers widen downstream
    pub depth_growth_rate: f32, // How quickly rivers deepen downstream
    pub erosion_strength: f32,  // How much rivers erode the terrain
    pub max_river_length: f32,  // Maximum length of a river
}

impl Default for RiverConfig {
    fn default() -> Self {
        Self {
            min_source_height: 0.7,
            meander_factor: 0.3,
            min_slope: 0.01,
            width_growth_rate: 0.1,
            depth_growth_rate: 0.05,
            erosion_strength: 0.2,
            max_river_length: 100.0,
        }
    }
}

// ---- Enums ----

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum BiomeType {
    Grass,
    Forest,
    Crops,
    Orchard,
    Water,
    Sand,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum FeatureType {
    Tree(TreeVariant),
    Bush(BushVariant),
    Flower(FlowerVariant),
    Rock,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum TreeVariant {
    EvergreenFir,
    WiltingFir,
    AppleTree,
    PrunedTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum BushVariant {
    GreenBushel,
    RipeBushel,
    DeadBushel,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum FlowerVariant {
    Single,
    Double,
    Quad,
    Cluster,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum FeatureVariant {
    Tree(TreeVariant),
    Bush(BushVariant),
    Flower(FlowerVariant),
    Rock,
}

// ---- Implementations ----

impl Default for TerrainConfig {
    fn default() -> Self {
        let mut feature_densities = HashMap::new();
        feature_densities.insert(FeatureType::Tree(TreeVariant::EvergreenFir), 0.6);
        feature_densities.insert(FeatureType::Tree(TreeVariant::AppleTree), 0.1);
        feature_densities.insert(FeatureType::Bush(BushVariant::GreenBushel), 0.2);
        feature_densities.insert(FeatureType::Flower(FlowerVariant::Single), 0.1);

        let mut biome_weights = HashMap::new();
        biome_weights.insert(BiomeType::Grass, 1.0);
        biome_weights.insert(BiomeType::Forest, 0.8);
        biome_weights.insert(BiomeType::Crops, 0.4);
        biome_weights.insert(BiomeType::Orchard, 0.3);
        biome_weights.insert(BiomeType::Water, 0.2);
        biome_weights.insert(BiomeType::Sand, 0.1);

        let river_config = RiverConfig::default();

        Self {
            noise_scale: 20.0,
            noise_octaves: 4,
            noise_persistence: 0.5,
            noise_lacunarity: 2.0,
            field_density: 0.3,
            biome_scale: 10.0,
            moisture_scale: 0.5,
            water_threshold: 0.7,
            beach_width: 0.025,
            feature_densities,
            biome_weights,
            river_config,
        }
    }
}

impl TerrainConfig {
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

    pub fn get_feature_density(&self, feature: FeatureType, biome: BiomeType) -> f32 {
        let base_density = self.feature_densities.get(&feature).unwrap_or(&0.0);

        match (feature, biome) {
            (FeatureType::Tree(_), BiomeType::Forest) => base_density * 1.5,
            (FeatureType::Tree(_), BiomeType::Orchard) => base_density * 1.2,
            (FeatureType::Bush(_), BiomeType::Crops) => base_density * 1.3,
            (FeatureType::Flower(_), BiomeType::Grass) => base_density * 1.1,
            _ => *base_density,
        }
    }
}

impl TerrainAssets {
    pub fn new() -> Self {
        Self {
            tile_texture: default(),
            feature_texture: default(),
            tile_layout: default(),
            feature_layout: default(),
            tile_mappings: HashMap::new(),
            feature_mappings: HashMap::new(),
        }
    }

    pub fn get_tile_sprite(&self, biome: BiomeType) -> Sprite {
        let index = self.tile_mappings.get(&biome).copied().unwrap_or(0);
        Sprite::from_atlas_image(
            self.tile_texture.clone(),
            TextureAtlas {
                layout: self.tile_layout.clone(),
                index,
            },
        )
    }

    pub fn get_feature_sprite(&self, feature: FeatureType) -> Sprite {
        let index = self.feature_mappings.get(&feature).copied().unwrap_or(0);
        Sprite::from_atlas_image(
            self.feature_texture.clone(),
            TextureAtlas {
                layout: self.feature_layout.clone(),
                index,
            },
        )
    }
}
