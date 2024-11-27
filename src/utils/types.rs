use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<Position> for Vector3<f64> {
    fn from(pos: Position) -> Self {
        Vector3::new(pos.x, pos.y, pos.z)
    }
}

impl From<Vector3<f64>> for Position {
    fn from(vec: Vector3<f64>) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AirData {
    pub true_airspeed: f64,
    pub calibrated_airspeed: f64,
    pub mach: f64,
    pub alpha: f64,
    pub beta: f64,
    pub dynamic_pressure: f64,
    pub static_pressure: f64,
    pub total_pressure: f64,
    pub density: f64,
    pub altitude: f64,
}
