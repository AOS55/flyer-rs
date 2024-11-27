use super::error::StateError;
use nalgebra::{UnitQuaternion, Vector3};
use std::any::Any;

pub trait SimState: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait StateManager {
    type State: SimState;

    fn get_state(&self) -> &Self::State;
    fn get_state_mut(&mut self) -> &mut Self::State;
    fn update_state(&mut self, new_state: Self::State) -> Result<(), StateError>;
}

pub trait SpatialOperations {
    fn position(&self) -> Vector3<f64>;
    fn velocity(&self) -> Vector3<f64>;
    fn attitude(&self) -> UnitQuaternion<f64>;
    fn angular_velocity(&self) -> Vector3<f64>;

    fn set_position(&mut self, position: Vector3<f64>) -> Result<(), StateError>;
    fn set_velocity(&mut self, velocity: Vector3<f64>) -> Result<(), StateError>;
    fn set_attitude(&mut self, attitude: UnitQuaternion<f64>) -> Result<(), StateError>;
    fn set_angular_velocity(&mut self, angular_velocity: Vector3<f64>) -> Result<(), StateError>;
}
