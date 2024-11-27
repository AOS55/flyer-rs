use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSettings {
    pub simulation_frequency: f64,
    pub policy_frequency: f64,
    pub render_frequency: f64,
    pub terrain_config: TerrainConfig,
    pub asset_config: AssetConfig,
    pub render_config: RenderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfig {
    pub seed: u64,
    pub area: (usize, usize),
    pub scaling: f32,
    pub water_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    pub assets_path: PathBuf,
    pub terrain_data_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub screen_width: f32,
    pub screen_height: f32,
    pub scale: f32,
    pub render_type: String,
}

impl Default for WorldSettings {
    fn default() -> Self {
        Self {
            simulation_frequency: 120.0,
            policy_frequency: 1.0,
            render_frequency: 0.01,
            terrain_config: TerrainConfig::default(),
            asset_config: AssetConfig::default(),
            render_config: RenderConfig::default(),
        }
    }
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            area: (100, 100),
            scaling: 1.0,
            water_present: false,
        }
    }
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            assets_path: PathBuf::from("assets"),
            terrain_data_path: PathBuf::from("terrain_data"),
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            screen_width: 1024.0,
            screen_height: 1024.0,
            scale: 25.0,
            render_type: "world".to_string(),
        }
    }
}
