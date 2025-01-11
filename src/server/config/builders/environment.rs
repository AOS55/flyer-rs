use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    resources::{AtmosphereConfig, AtmosphereType, EnvironmentConfig, WindConfig},
    server::config::errors::ConfigError,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfigBuilder {
    wind_builder: Option<WindConfigBuilder>,
    atmosphere_builder: Option<AtmosphereConfigBuilder>,
    seed: Option<u64>,
}

impl EnvironmentConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    // pub fn with_seed(mut self, master_seed: u64) -> Self {
    //     use std::hash::{Hash, Hasher};
    //     let mut hasher = std::collections::hash_map::DefaultHasher::new();
    //     master_seed.hash(&mut hasher);
    //     "environment".hash(&mut hasher);
    //     self.seed = Some(hasher.finish());
    //     self
    // }

    pub fn wind_config(mut self, builder: WindConfigBuilder) -> Self {
        self.wind_builder = Some(builder);
        self
    }

    pub fn atmosphere_config(mut self, builder: AtmosphereConfigBuilder) -> Self {
        self.atmosphere_builder = Some(builder);
        self
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(wind_config) = value.get("wind_model_config") {
            builder = builder.wind_config(WindConfigBuilder::from_json(wind_config)?);
        }

        if let Some(atmosphere_config) = value.get("atmosphere_config") {
            builder =
                builder.atmosphere_config(AtmosphereConfigBuilder::from_json(atmosphere_config)?);
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<EnvironmentConfig, ConfigError> {
        let mut wind_builder = self.wind_builder.unwrap_or_default();
        let mut atm_builder = self.atmosphere_builder.unwrap_or_default();

        if let Some(seed) = self.seed {
            wind_builder = wind_builder.with_seed(seed);
            atm_builder = atm_builder.with_seed(seed);
        }

        Ok(EnvironmentConfig {
            wind_model_config: wind_builder.build()?,
            atmosphere_config: atm_builder.build()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindConfigBuilder {
    wind_type: Option<String>,
    velocity: Option<Vector3<f64>>,
    // Logarithmic model parameters
    d: Option<f64>,
    z0: Option<f64>,
    u_star: Option<f64>,
    bearing: Option<f64>,
    // Power law parameters
    u_r: Option<f64>,
    z_r: Option<f64>,
    alpha: Option<f64>,

    // Generation seed
    seed: Option<u64>,
}

impl Default for WindConfigBuilder {
    fn default() -> Self {
        Self {
            wind_type: Some("Constant".to_string()),
            velocity: Some(Vector3::new(0.0, 0.0, 0.0)),
            d: None,
            z0: None,
            u_star: None,
            bearing: None,
            u_r: None,
            z_r: None,
            alpha: None,
            seed: None,
        }
    }
}

impl WindConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(mut self, master_seed: u64) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        master_seed.hash(&mut hasher);
        "wind".hash(&mut hasher);
        self.seed = Some(hasher.finish());
        self
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        let Some(wind_type) = value.get("type").and_then(|v| v.as_str()) else {
            return Ok(builder);
        };

        let wind_type = wind_type.to_string();
        builder.wind_type = Some(wind_type.clone());

        match wind_type.as_str() {
            "Constant" => {
                if let Some(velocity) = value.get("velocity") {
                    if let (Some(x), Some(y), Some(z)) = (
                        velocity.get(0).and_then(|v| v.as_f64()),
                        velocity.get(1).and_then(|v| v.as_f64()),
                        velocity.get(2).and_then(|v| v.as_f64()),
                    ) {
                        builder.velocity = Some(Vector3::new(x, y, z));
                    }
                }
            }
            "Logarithmic" => {
                if let Some(d) = value.get("d").and_then(|v| v.as_f64()) {
                    builder.d = Some(d);
                }
                if let Some(z0) = value.get("z0").and_then(|v| v.as_f64()) {
                    builder.z0 = Some(z0);
                }
                if let Some(u_star) = value.get("u_star").and_then(|v| v.as_f64()) {
                    builder.u_star = Some(u_star);
                }
                if let Some(bearing) = value.get("bearing").and_then(|v| v.as_f64()) {
                    builder.bearing = Some(bearing);
                }
            }
            "PowerLaw" => {
                if let Some(u_r) = value.get("u_r").and_then(|v| v.as_f64()) {
                    builder.u_r = Some(u_r);
                }
                if let Some(z_r) = value.get("z_r").and_then(|v| v.as_f64()) {
                    builder.z_r = Some(z_r);
                }
                if let Some(alpha) = value.get("alpha").and_then(|v| v.as_f64()) {
                    builder.alpha = Some(alpha);
                }
                if let Some(bearing) = value.get("bearing").and_then(|v| v.as_f64()) {
                    builder.bearing = Some(bearing);
                }
            }
            _ => {
                return Err(ConfigError::InvalidParameter {
                    name: "wind_type".into(),
                    value: wind_type,
                })
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<WindConfig, ConfigError> {
        match self.wind_type.as_deref() {
            Some("Constant") => {
                let velocity = self.velocity.ok_or_else(|| {
                    ConfigError::MissingRequired("velocity for Constant wind".into())
                })?;
                Ok(WindConfig::Constant { velocity })
            }
            Some("Logarithmic") => Ok(WindConfig::Logarithmic {
                d: self.d.unwrap_or(0.0),
                z0: self.z0.unwrap_or(0.0),
                u_star: self.u_star.unwrap_or(0.0),
                bearing: self.bearing.unwrap_or(0.0),
            }),
            Some("PowerLaw") => Ok(WindConfig::PowerLaw {
                u_r: self.u_r.unwrap_or(0.0),
                z_r: self.z_r.unwrap_or(0.0),
                alpha: self.alpha.unwrap_or(0.0),
                bearing: self.bearing.unwrap_or(0.0),
            }),
            _ => Err(ConfigError::InvalidParameter {
                name: "wind_type".into(),
                value: self.wind_type.unwrap_or_default(),
            }),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphereConfigBuilder {
    model_type: Option<AtmosphereType>,
    sea_level_density: Option<f64>,
    sea_level_temperature: Option<f64>,
    seed: Option<u64>,
}

impl AtmosphereConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(mut self, master_seed: u64) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        master_seed.hash(&mut hasher);
        "atmosphere".hash(&mut hasher);
        self.seed = Some(hasher.finish());
        self
    }
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(model_type) = value.get("model_type").and_then(|v| v.as_str()) {
            builder.model_type = Some(match model_type {
                "Constant" => AtmosphereType::Constant,
                _ => AtmosphereType::Standard,
            });
        }

        if let Some(density) = value.get("sea_level_density").and_then(|v| v.as_f64()) {
            builder.sea_level_density = Some(density);
        }

        if let Some(temperature) = value.get("sea_level_temperature").and_then(|v| v.as_f64()) {
            builder.sea_level_temperature = Some(temperature);
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<AtmosphereConfig, ConfigError> {
        Ok(AtmosphereConfig {
            model_type: self.model_type.unwrap_or(AtmosphereType::Standard),
            sea_level_density: self.sea_level_density.unwrap_or(1.225),
            sea_level_temperature: self.sea_level_temperature.unwrap_or(288.15),
        })
    }
}
