use nalgebra::{UnitQuaternion, Vector3};

use crate::physics::aerso::{
    AersoConfig, AersoPhysics, AersoPhysicsState, AtmosphereModel, WindModelConfig,
};
use crate::rendering::RenderError;
use crate::state::{StateError, StateManager};
use crate::utils::errors::SimError;
use crate::vehicles::Vehicle;

use super::config::AircraftConfig;
use super::state::AircraftState;

pub struct Aircraft {
    state: AircraftState,
    config: AircraftConfig,
    physics: AersoPhysics,
}

impl Aircraft {
    pub fn new(config: AircraftConfig) -> Result<Self, SimError> {
        // Convert AircraftConfig to AersoConfig
        let aerso_config = AersoConfig {
            mass: config.mass,
            inertia: config.inertia,
            aero_coefficients: config.aero.into(),
            engine_params: config.propulsion.into(),
            atmosphere_model: AtmosphereModel::Standard,
            wind_model: WindModelConfig::Constant(Vector3::zeros()),
        };

        // Initialize with default state and physics
        let state = AircraftState::default();
        let physics = AersoPhysics::new(&config)?;

        Ok(Self {
            state,
            config,
            physics,
        })
    }

    pub fn new_with_state(
        config: AircraftConfig,
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        attitude: UnitQuaternion<f64>,
        angular_velocity: Vector3<f64>,
    ) -> Result<Self, SimError> {
        let mut aircraft = AircraftState::default();
        aircraft.state.spatial.position = position;
        aircraft.state.spatial.velocity = velocity;
        aircraft.state.spatial.attitude = attitude;
        aircraft.state.spatial.angular_velocity = angular_velocity;

        Ok(aircraft)
    }

    // Convert AircraftState to AersoPhysicsState
    fn convert_to_physics_state(&self) -> AersoPhysicsState {
        let mut physics_state = AersoPhysicsState::new(self.config.mass)?;
        physics_state.spatial = self.state.spatial.clone();
        physics_state.controls = self.state.controls.clone().into();
        physics_state
    }
}

impl StateManager for Aircraft {
    type State = AircraftState;

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }

    fn update_state(&mut self, new_state: Self::State) -> Result<(), StateError> {
        // Basic validation of new state
        new_state
            .validate()
            .map_err(|e| StateError::ValidationFailed(e.to_string()))?;

        self.state = new_state;
        Ok(())
    }
}

impl Vehicle for Aircraft {
    fn update(&mut self, dt: f64) -> Result<(), SimError> {
        // Convert current state to physics state
        let mut physics_state = self.convert_to_physics_state();

        // Step physics simulation
        self.physics
            .step(&mut physics_state, dt)
            .map_err(|e| SimError::PhysicsError(e.to_string()))?;

        // Update aircraft state from physics
        self.state.update_from_physics(
            physics_state.spatial.position,
            physics_state.spatial.velocity,
            physics_state.spatial.attitude,
            physics_state.spatial.angular_velocity,
        );

        // Update controls with limits
        self.apply_control_limits();

        Ok(())
    }

    fn render(&self) -> Result<(), RenderError> {
        // Implement rendering logic or return Ok if not needed
        Ok(())
    }
}

// Private helper methods
impl Aircraft {
    fn apply_control_limits(&mut self) {
        let controls = &mut self.state.controls;

        // Clamp control surfaces to their limits
        controls.aileron = controls.aileron.clamp(-1.0, 1.0);
        controls.elevator = controls.elevator.clamp(-1.0, 1.0);
        controls.rudder = controls.rudder.clamp(-1.0, 1.0);
        controls.throttle = controls.throttle.clamp(0.0, 1.0);
        controls.flaps = controls.flaps.clamp(0.0, 1.0);
        controls.brake = controls.brake.clamp(0.0, 1.0);
    }
}

impl Default for Aircraft {
    fn default() -> Self {
        let config = AircraftConfig::default();
        Self::new(config).expect("Failed to create default aircraft")
    }
}
