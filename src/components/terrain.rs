use crate::ecs::component::Component;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;

/// Represents a single terrain tile's data
#[derive(Clone, Serialize, Deserialize)]
pub struct TerrainTile {
    pub name: String,
    pub asset: String,
    pub pos: Vec2,
}

/// Represents a static object on the terrain
#[derive(Clone, Serialize, Deserialize)]
pub struct TerrainObject {
    pub name: String,
    pub asset: String,
    pub pos: Vec2,
}

/// Terrain configuration data
#[derive(Clone)]
pub struct TerrainConfig {
    pub name: String,
    pub field_density: f32,
    pub land_types: Vec<String>,
    pub water_cutoff: f32,
    pub beach_thickness: f32,
}

/// Main terrain component storing all terrain-related data
#[derive(Clone)]
pub struct TerrainComponent {
    /// Configuration for the terrain
    pub config: TerrainConfig,
    /// List of terrain tiles
    pub tiles: Vec<TerrainTile>,
    /// List of static objects on the terrain
    pub objects: Vec<TerrainObject>,
    /// Dimensions of the terrain area
    pub area: Vec<usize>,
    /// Terrain scaling factor
    pub scaling: f32,
    /// Whether water features are present
    pub water_present: bool,
    /// Seed for terrain generation
    pub seed: u64,
    /// Raw terrain data for height/etc calculations
    pub terrain_data: Option<Arc<Vec<u8>>>,
}

impl Component for TerrainComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for TerrainComponent {
    fn default() -> Self {
        Self {
            config: TerrainConfig {
                name: "default".to_string(),
                field_density: 0.001,
                land_types: vec!["grass".to_string(), "forest".to_string()],
                water_cutoff: -0.1,
                beach_thickness: 0.04,
            },
            tiles: Vec::new(),
            objects: Vec::new(),
            area: vec![100, 100],
            scaling: 1.0,
            water_present: false,
            seed: 0,
            terrain_data: None,
        }
    }
}

impl TerrainComponent {
    pub fn new(
        config: TerrainConfig,
        area: Vec<usize>,
        scaling: f32,
        water_present: bool,
        seed: u64,
    ) -> Self {
        Self {
            config,
            tiles: Vec::new(),
            objects: Vec::new(),
            area,
            scaling,
            water_present,
            seed,
            terrain_data: None,
        }
    }

    /// Get terrain dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.area[0], self.area[1])
    }
}
