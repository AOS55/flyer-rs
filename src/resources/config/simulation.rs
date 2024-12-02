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

mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = SimulationConfig::default();
        assert_eq!(config.render.screen_width, 1920);
        assert_eq!(config.render.screen_height, 1080);
        assert_eq!(config.physics.time_step, 1.0 / 120.0);
        assert!(config.assets.base_path.to_str().unwrap().contains("assets"));
    }

    #[test]
    fn test_config_save_load() -> Result<(), Box<dyn std::error::Error>> {
        let config = SimulationConfig::default();
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path().to_str().unwrap();

        // Test saving
        config.save(path)?;
        assert!(fs::metadata(path).is_ok());

        // Test loading
        let loaded_config = SimulationConfig::load(path)?;
        assert_eq!(
            loaded_config.render.screen_width,
            config.render.screen_width
        );
        assert_eq!(loaded_config.physics.time_step, config.physics.time_step);

        Ok(())
    }

    #[test]
    fn test_invalid_config_load() {
        let result = SimulationConfig::load("nonexistent_file.yaml");
        assert!(result.is_err());
    }
}
