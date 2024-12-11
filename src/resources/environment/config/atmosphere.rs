use serde::{Deserialize, Serialize};

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

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self {
            model_type: AtmosphereType::Standard,
            sea_level_density: 1.225,
            sea_level_temperature: 288.15,
        }
    }
}
