use super::{asset::AssetConfig, physics::PhysicsConfig, render::RenderConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub physics: PhysicsConfig,
    pub render: RenderConfig,
    pub assets: AssetConfig,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            physics: PhysicsConfig {
                time_step: 1.0 / 120.0,
                max_substeps: 4,
                gravity: 9.81,
                air_density: 1.225,
            },
            render: RenderConfig {
                screen_width: 1920,
                screen_height: 1080,
                vsync: true,
                fov: 75.0,
                draw_distance: 10000.0,
            },
            assets: AssetConfig {
                base_path: PathBuf::from("assets"),
                models_path: PathBuf::from("assets/models"),
                textures_path: PathBuf::from("assets/textures"),
            },
        }
    }
}

impl SimulationConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let config = serde_yaml::from_reader(file)?;
        Ok(config)
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        serde_yaml::to_writer(file, self)?;
        Ok(())
    }
}
