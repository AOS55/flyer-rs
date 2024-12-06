use bevy::prelude::*;
use nalgebra::Vector3;

/// Configuration for the physics system
#[derive(Resource)]
pub struct PhysicsConfig {
    // Integration parameters
    pub max_velocity: f64,
    pub max_angular_velocity: f64,
    pub timestep: f64,

    // Force parameters
    pub gravity: Vector3<f64>,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            max_velocity: 200.0,        // m/s
            max_angular_velocity: 10.0, // rad/s
            timestep: 1.0 / 120.0,      // 120 Hz
            gravity: Vector3::new(0.0, 0.0, -9.81),
        }
    }
}
