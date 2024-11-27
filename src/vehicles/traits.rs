use crate::physics::traits::PhysicsModel;
use crate::utils::errors::SimError;
use nalgebra::{UnitQuaternion, Vector3};

pub trait Vehicle {
    type State;
    type Controls;
    type Config;

    fn new(config: Self::Config) -> Result<Self, SimError>
    where
        Self: Sized;
    fn update_state(&mut self, physics: &dyn PhysicsModel);
    fn get_state(&self) -> &Self::State;
    fn set_controls(&mut self, controls: Self::Controls);
    fn reset(&mut self, state: Self::State);
}

pub trait VehicleState {
    fn position(&self) -> Vector3<f64>;
    fn velocity(&self) -> Vector3<f64>;
    fn attitude(&self) -> UnitQuaternion<f64>;
    fn rates(&self) -> Vector3<f64>;
}

pub trait Controls {
    fn validate(&self) -> Result<(), SimError>;
    fn interpolate(&self, other: &Self, factor: f64) -> Self;
}
