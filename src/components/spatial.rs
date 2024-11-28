use crate::ecs::component::Component;
use nalgebra::{UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// Component for storing spatial state of an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Component for SpatialComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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
}
