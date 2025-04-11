use aerso::density_models::StandardDensity;
use aerso::wind_models::{ConstantWind, LogWind, PowerWind};
use aerso::{DensityModel, WindModel};
use bevy::prelude::*;
use nalgebra::Vector3;

use crate::resources::environment::{EnvironmentConfig, WindConfig};

#[derive(Resource)]
pub struct EnvironmentModel {
    wind_model: Box<dyn WindModel<f64> + Send + Sync>,
    density_model: Box<dyn DensityModel<f64> + Send + Sync>,
}

impl EnvironmentModel {
    pub fn new(config: &EnvironmentConfig) -> Self {
        let wind_model = match &config.wind_model_config {
            WindConfig::Constant { velocity } => {
                Box::new(ConstantWind::new(*velocity)) as Box<dyn WindModel<f64> + Send + Sync>
            }
            WindConfig::Logarithmic {
                d,
                z0,
                u_star,
                bearing,
            } => Box::new(LogWind::new(*d, *z0, *u_star, *bearing))
                as Box<dyn WindModel<f64> + Send + Sync>,
            WindConfig::PowerLaw {
                u_r,
                z_r,
                bearing,
                alpha,
            } => Box::new(PowerWind::new_with_alpha(*u_r, *z_r, *bearing, *alpha))
                as Box<dyn WindModel<f64> + Send + Sync>,
        };

        let density_model = Box::new(StandardDensity) as Box<dyn DensityModel<f64> + Send + Sync>;

        Self {
            wind_model,
            density_model,
        }
    }

    pub fn get_wind(&self, position: &Vector3<f64>) -> Vector3<f64> {
        let ned_position: Vector3<f64> = Vector3::new(position.x, position.y, -position.z);
        self.wind_model.get_wind(&ned_position)
    }

    pub fn get_density(&self, position: &Vector3<f64>) -> f64 {
        let ned_position: Vector3<f64> = Vector3::new(position.x, position.y, -position.z);
        self.density_model.get_density(&ned_position)
    }

    pub fn get_density_at_altitude(&self, altitude: f64) -> f64 {
        let ned_position: Vector3<f64> = Vector3::new(0.0, 0.0, -altitude);
        self.density_model.get_density(&ned_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::environment::AtmosphereConfig;

    #[test]
    fn test_constant_wind() {
        let config = EnvironmentConfig {
            wind_model_config: WindConfig::Constant {
                velocity: Vector3::new(1.0, 0.0, 0.0),
            },
            atmosphere_config: AtmosphereConfig::default(),
        };

        let env = EnvironmentModel::new(&config);
        let position = Vector3::new(0.0, 0.0, 0.0);
        let wind = env.get_wind(&position);

        assert_eq!(wind, Vector3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_density_model() {
        let config = EnvironmentConfig::default();
        let env = EnvironmentModel::new(&config);

        let ground_position = Vector3::new(0.0, 0.0, 0.0);
        let altitude_position = Vector3::new(0.0, 0.0, 1000.0);

        let ground_density = env.get_density(&ground_position);
        let altitude_density = env.get_density(&altitude_position);

        assert!(ground_density > altitude_density);
    }
}
