use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub time_step: f64,
    pub max_substeps: u32,
    pub gravity: f64,
    pub air_density: f64,
}
