use crate::physics::components::ForceSystem;
use crate::physics::error::PhysicsError;
use crate::state::SimState;
use nalgebra::Vector3;

pub trait PhysicsState: SimState {
    fn mass(&self) -> f64;
    fn add_force(&mut self, force: Vector3<f64>);
    fn add_moment(&mut self, moment: Vector3<f64>);
    fn clear_forces(&mut self);
}

pub trait PhysicsModel {
    type State: PhysicsState;
    type Config;

    fn new(config: Self::Config) -> Result<Self, PhysicsError>
    where
        Self: Sized;

    fn step(&mut self, state: &mut Self::State, dt: f64) -> Result<(), PhysicsError>;
    fn reset(&mut self);
    fn get_force_system(&self) -> &ForceSystem;
}
