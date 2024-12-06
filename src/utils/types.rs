use bevy::reflect::{FromReflect, Reflect};
use glam::Vec2;
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

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ReflectableVec2 {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2> for ReflectableVec2 {
    fn from(vec: Vec2) -> Self {
        Self { x: vec.x, y: vec.y }
    }
}

impl From<ReflectableVec2> for Vec2 {
    fn from(reflectable: ReflectableVec2) -> Self {
        Vec2::new(reflectable.x, reflectable.y)
    }
}
