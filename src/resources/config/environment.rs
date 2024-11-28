use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

pub struct EnvironmentConfig {
    pub wind_model_config: WindModelConfig,
    pub atmosphere_config: AtmosphereConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindModelConfig {
    Constant {
        velocity: Vector3<f64>,
    },
    Logarithmic {
        d: f64,
        z0: f64,
        u_star: f64,
        bearing: f64,
    },
    PowerLaw {
        u_r: f64,
        z_r: f64,
        bearing: f64,
        alpha: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphereConfig {
    pub model_type: AtmosphereType,
    pub sea_level_density: f64,
    pub sea_level_temperature: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtmosphereType {
    Constant,
    Standard,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            wind_model_config: WindModelConfig::Constant {
                velocity: Vector3::new(0.0, 0.0, 0.0),
            },
            atmosphere_config: AtmosphereConfig::default(),
        }
    }
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self {
            model_type: AtmosphereType::Standard,
            sea_level_density: 1.225,
            sea_level_temperature: 288.15,
        }
    }
}
