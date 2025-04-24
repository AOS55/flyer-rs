use bevy::prelude::Component;
use nalgebra::{UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// Component for storing spatial state of an entity
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpatialComponent {
    /// Position in world space [m]
    pub position: Vector3<f64>,

    /// Linear velocity in world space [m/s]
    pub velocity: Vector3<f64>,

    /// Attitude quaternion (rotation from body to world frame)
    pub attitude: UnitQuaternion<f64>,

    /// Angular velocity in body frame [rad/s]
    pub angular_velocity: Vector3<f64>,
}

impl Default for SpatialComponent {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        }
    }
}

impl SpatialComponent {
    /// Create a new spatial component with initial values
    pub fn new(
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        attitude: UnitQuaternion<f64>,
        angular_velocity: Vector3<f64>,
    ) -> Self {
        Self {
            position,
            velocity,
            attitude,
            angular_velocity,
        }
    }

    /// Create a new spatial component at a specific position
    pub fn at_position(position: Vector3<f64>) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create level position and airspeed
    pub fn at_position_and_airspeed(position: Vector3<f64>, speed: f64, heading: f64) -> Self {
        Self {
            position,
            velocity: Vector3::new(speed * heading.cos(), speed * heading.sin(), 0.0),
            ..Default::default()
        }
    }
}
