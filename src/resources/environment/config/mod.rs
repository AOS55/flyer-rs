mod atmosphere;
mod wind;

pub use atmosphere::{AtmosphereConfig, AtmosphereType};
pub use wind::WindConfig;

use nalgebra::Vector3;

pub struct EnvironmentConfig {
    pub wind_model_config: WindConfig,
    pub atmosphere_config: AtmosphereConfig,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            wind_model_config: WindConfig::Constant {
                velocity: Vector3::new(0.0, 0.0, 0.0),
            },
            atmosphere_config: AtmosphereConfig::default(),
        }
    }
}
