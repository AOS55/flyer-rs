use crate::utils::errors::SimError;
use crate::vehicles::traits::VehicleState;
use aerso::types::{Force, Vector3};
use aerso::{AeroEffect, AirState}; // Import necessary types

pub trait PhysicsModel {
    type State;
    type Parameters;

    fn new(params: Self::Parameters) -> Result<Self, SimError>
    where
        Self: Sized;
    fn step(&mut self, state: &mut dyn VehicleState, dt: f64);
    fn get_timestep(&self) -> f64;
    fn reset(&mut self);
    fn get_forces(&self) -> Vec<Force>;
    fn get_accelerations(&self) -> Vector3<f64>;
}

pub trait SimplifiedPhysics: PhysicsModel {
    fn update_kinematics(&mut self, state: &mut dyn VehicleState, dt: f64);
}

pub trait AerodynamicsModel: PhysicsModel {
    fn get_aero_forces(&self, state: &dyn VehicleState) -> Vec<Force>;
    fn get_air_data(&self, state: &dyn VehicleState) -> AirState;
}
