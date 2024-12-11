use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindConfig {
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
