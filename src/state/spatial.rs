use nalgebra::{UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

use super::error::StateError;
use super::traits::SpatialOperations;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialState {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub attitude: UnitQuaternion<f64>,
    pub angular_velocity: Vector3<f64>,
}

impl Default for SpatialState {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        }
    }
}

impl SpatialOperations for SpatialState {
    fn position(&self) -> Vector3<f64> {
        self.position
    }

    fn velocity(&self) -> Vector3<f64> {
        self.velocity
    }

    fn attitude(&self) -> UnitQuaternion<f64> {
        self.attitude
    }

    fn angular_velocity(&self) -> Vector3<f64> {
        self.angular_velocity
    }

    fn set_position(&mut self, position: Vector3<f64>) -> Result<(), StateError> {
        self.position = position;
        Ok(())
    }

    fn set_velocity(&mut self, velocity: Vector3<f64>) -> Result<(), StateError> {
        self.velocity = velocity;
        Ok(())
    }

    fn set_attitude(&mut self, attitude: UnitQuaternion<f64>) -> Result<(), StateError> {
        self.attitude = attitude;
        Ok(())
    }

    fn set_angular_velocity(&mut self, angular_velocity: Vector3<f64>) -> Result<(), StateError> {
        self.angular_velocity = angular_velocity;
        Ok(())
    }
}
