mod atmosphere;
mod wind;

pub use atmosphere::{AtmosphereConfig, AtmosphereType};
pub use wind::WindConfig;

use nalgebra::Vector3;

#[derive(Debug, Clone)]
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

impl EnvironmentConfig {
    pub fn new(wind_config: WindConfig, atmosphere_config: AtmosphereConfig) -> Self {
        Self {
            wind_model_config: wind_config,
            atmosphere_config,
        }
    }

    pub fn with_constant_wind(wind_speed: f64, wind_direction: f64) -> Self {
        let angle_rad = wind_direction.to_radians();
        let velocity = Vector3::new(
            wind_speed * angle_rad.sin(),
            wind_speed * angle_rad.cos(),
            0.0,
        );

        Self {
            wind_model_config: WindConfig::Constant { velocity },
            atmosphere_config: AtmosphereConfig::default(),
        }
    }

    pub fn with_logarithmic_wind(d: f64, z0: f64, u_star: f64, bearing: f64) -> Self {
        Self {
            wind_model_config: WindConfig::Logarithmic {
                d,
                z0,
                u_star,
                bearing,
            },
            atmosphere_config: AtmosphereConfig::default(),
        }
    }

    pub fn with_power_law_wind(u_r: f64, z_r: f64, bearing: f64, alpha: f64) -> Self {
        Self {
            wind_model_config: WindConfig::PowerLaw {
                u_r,
                z_r,
                bearing,
                alpha,
            },
            atmosphere_config: AtmosphereConfig::default(),
        }
    }
}
