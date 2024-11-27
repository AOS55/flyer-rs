use super::spatial::SpatialState;
use super::traits::SimState;
use crate::physics::components::{Force, ForceSystem, Moment};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsState {
    pub spatial: SpatialState,
    pub mass: f64,
    #[serde(skip)]
    pub force_system: ForceSystem,
}

impl SimState for PhysicsState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            spatial: SpatialState::default(),
            mass: 1.0,
            force_system: ForceSystem::new(),
        }
    }
}

impl PhysicsState {
    /// Create a new PhysicsState with given mass
    pub fn new(mass: f64) -> Self {
        Self {
            spatial: SpatialState::default(),
            mass,
            force_system: ForceSystem::new(),
        }
    }

    /// Create a new PhysicsState with custom spatial state and mass
    pub fn with_spatial(spatial: SpatialState, mass: f64) -> Self {
        Self {
            spatial,
            mass,
            force_system: ForceSystem::new(),
        }
    }

    /// Add a force to the system
    pub fn add_force(&mut self, force: Force) {
        self.force_system.add_force(force);
    }

    /// Add a moment to the system
    pub fn add_moment(&mut self, moment: Moment) {
        self.force_system.add_moment(moment);
    }

    /// Clear all forces and moments
    pub fn clear_forces(&mut self) {
        self.force_system.clear();
    }

    /// Get net force in inertial frame
    pub fn net_force(&self) -> Vector3<f64> {
        self.force_system.net_force()
    }

    /// Get net moment in body frame
    pub fn net_moment(&self) -> Vector3<f64> {
        self.force_system.net_moment()
    }

    /// Reset the state to default values while preserving mass
    pub fn reset(&mut self) {
        let mass = self.mass;
        *self = Self::default();
        self.mass = mass;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = PhysicsState::default();
        assert_eq!(state.mass, 1.0);
        assert!(state.net_force().norm() < 1e-10);
        assert!(state.spatial.position.norm() < 1e-10);
    }

    #[test]
    fn test_serialization() {
        let state = PhysicsState::new(2.0);

        // Test serialization
        let serialized = serde_json::to_string(&state).unwrap();

        // Test deserialization
        let deserialized: PhysicsState = serde_json::from_str(&serialized).unwrap();

        assert_eq!(state.mass, deserialized.mass);
        assert_eq!(state.spatial.position, deserialized.spatial.position);
    }

    #[test]
    fn test_reset() {
        let mut state = PhysicsState::new(2.0);
        state.spatial.position = Vector3::new(1.0, 1.0, 1.0);

        state.reset();

        assert_eq!(state.mass, 2.0); // Mass should be preserved
        assert!(state.spatial.position.norm() < 1e-10); // Position should be reset
    }

    #[test]
    fn test_with_spatial() {
        let spatial = SpatialState {
            position: Vector3::new(1.0, 0.0, 0.0),
            ..SpatialState::default()
        };

        let state = PhysicsState::with_spatial(spatial, 2.0);

        assert_eq!(state.mass, 2.0);
        assert_eq!(state.spatial.position.x, 1.0);
    }
}
