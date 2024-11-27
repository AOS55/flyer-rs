use crate::state::{SimState, SpatialState};
use crate::utils::constants::*;
use crate::utils::errors::SimError;
use nalgebra::{UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftState {
    pub spatial: SpatialState,
    pub controls: AircraftControls,
    pub air_data: AirData,
    pub system_state: SystemState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftControls {
    pub aileron: f64,
    pub elevator: f64,
    pub rudder: f64,
    pub throttle: f64,
    pub flaps: f64,
    pub gear: bool,
    pub brake: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AirData {
    pub true_airspeed: f64,
    pub calibrated_airspeed: f64,
    pub mach: f64,
    pub alpha: f64,
    pub beta: f64,
    pub dynamic_pressure: f64,
    pub static_pressure: f64,
    pub total_pressure: f64,
    pub density: f64,
    pub altitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub engine_rpm: f64,
    pub engine_thrust: f64,
    pub fuel_flow: f64,
    pub surface_temperatures: SurfaceTemperatures,
    pub load_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceTemperatures {
    pub engine: f64,
    pub brakes: f64,
}

impl Default for AircraftState {
    fn default() -> Self {
        Self {
            spatial: SpatialState::default(),
            controls: AircraftControls::default(),
            air_data: AirData::default(),
            system_state: SystemState::default(),
        }
    }
}

impl SimState for AircraftState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl AircraftState {
    pub fn new(position: Vector3<f64>, attitude: UnitQuaternion<f64>) -> Self {
        let mut state = Self::default();
        state.spatial.position = position;
        state.spatial.attitude = attitude;
        state
    }

    pub fn validate(&self) -> Result<(), SimError> {
        self.validate_controls()?;
        self.validate_air_data()?;
        self.validate_system_state()?;
        Ok(())
    }

    pub fn update_from_physics(
        &mut self,
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        attitude: UnitQuaternion<f64>,
        angular_velocity: Vector3<f64>,
    ) {
        self.spatial.position = position;
        self.spatial.velocity = velocity;
        self.spatial.attitude = attitude;
        self.spatial.angular_velocity = angular_velocity;
    }

    fn validate_controls(&self) -> Result<(), SimError> {
        let controls = &self.controls;

        if !(-1.0..=1.0).contains(&controls.aileron)
            || !(-1.0..=1.0).contains(&controls.elevator)
            || !(-1.0..=1.0).contains(&controls.rudder)
            || !(0.0..=1.0).contains(&controls.throttle)
            || !(0.0..=1.0).contains(&controls.flaps)
            || !(0.0..=1.0).contains(&controls.brake)
        {
            return Err(SimError::InvalidControl(
                "Control surface deflection out of bounds".into(),
            ));
        }
        Ok(())
    }

    fn validate_air_data(&self) -> Result<(), SimError> {
        let air = &self.air_data;

        if air.alpha.abs() > MAX_ANGLE_OF_ATTACK
            || air.beta.abs() > MAX_SIDESLIP
            || air.true_airspeed < 0.0
            || air.density < 0.0
        {
            return Err(SimError::PhysicsError("Invalid air data parameters".into()));
        }
        Ok(())
    }

    fn validate_system_state(&self) -> Result<(), SimError> {
        let system = &self.system_state;

        if system.load_factor > MAX_LOAD_FACTOR
            || system.load_factor < MIN_LOAD_FACTOR
            || system.engine_rpm < 0.0
            || system.fuel_flow < 0.0
        {
            return Err(SimError::StateError(
                "Invalid system state parameters".into(),
            ));
        }
        Ok(())
    }
}

impl Default for AircraftControls {
    fn default() -> Self {
        Self {
            aileron: 0.0,
            elevator: 0.0,
            rudder: 0.0,
            throttle: 0.0,
            flaps: 0.0,
            gear: false,
            brake: 0.0,
        }
    }
}

impl Default for AirData {
    fn default() -> Self {
        Self {
            true_airspeed: 0.0,
            calibrated_airspeed: 0.0,
            mach: 0.0,
            alpha: 0.0,
            beta: 0.0,
            dynamic_pressure: 0.0,
            static_pressure: ISA_SEA_LEVEL_PRESSURE,
            total_pressure: ISA_SEA_LEVEL_PRESSURE,
            density: 1.225,
            altitude: 0.0,
        }
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            engine_rpm: 0.0,
            engine_thrust: 0.0,
            fuel_flow: 0.0,
            surface_temperatures: SurfaceTemperatures {
                engine: ISA_SEA_LEVEL_TEMP,
                brakes: ISA_SEA_LEVEL_TEMP,
            },
            load_factor: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::math::deg_to_rad;

    #[test]
    fn test_aircraft_state_validation() {
        let mut state = AircraftState::default();
        assert!(state.validate().is_ok());

        state.controls.aileron = 1.5;
        assert!(state.validate().is_err());
    }

    #[test]
    fn test_air_data_validation() {
        let mut state = AircraftState::default();
        state.air_data.alpha = deg_to_rad(25.0);
        assert!(state.validate().is_ok());

        state.air_data.alpha = deg_to_rad(35.0);
        assert!(state.validate().is_err());
    }

    #[test]
    fn test_system_state_validation() {
        let mut state = AircraftState::default();
        state.system_state.load_factor = 5.0;
        assert!(state.validate().is_ok());

        state.system_state.load_factor = 12.0;
        assert!(state.validate().is_err());
    }
}
