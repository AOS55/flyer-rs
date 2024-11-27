pub mod camera;
pub mod terrain;

use crate::utils::errors::SimError;
use crate::world::core::SimulationState;

/// Base trait for all simulation systems
pub trait System: Send + Sync {
    fn update(&mut self, state: &mut SimulationState, dt: f64) -> Result<(), SimError>;
    fn reset(&mut self);
}

/// Marker trait for systems that handle physics
pub trait PhysicsSystem: System {}

/// Marker trait for systems that handle rendering
pub trait RenderSystem: System {}

/// Marker trait for systems that handle terrain
pub trait TerrainSystem: System {}
