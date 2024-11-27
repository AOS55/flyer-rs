use nalgebra::{UnitQuaternion, Vector3};
use std::any::Any;

pub use self::error::StateError;

mod error;

pub trait VehicleState: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn position(&self) -> Vector3<f64>;
    fn velocity(&self) -> Vector3<f64>;
    fn attitude(&self) -> UnitQuaternion<f64>;
    fn rates(&self) -> Vector3<f64>;
}

#[derive(Debug, Clone)]
pub struct SpatialState {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub attitude: UnitQuaternion<f64>,
    pub rates: Vector3<f64>,
}

impl Default for SpatialState {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            rates: Vector3::zeros(),
        }
    }
}

pub trait StateManager {
    type State: VehicleState;

    fn get_state(&self) -> &Self::State;
    fn get_state_mut(&mut self) -> &mut Self::State;
    fn update_state(&mut self, new_state: Self::State) -> Result<(), StateError>;
}
