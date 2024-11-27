use crate::physics::components::{Force, ForceSystem, ForceType, Moment};
use crate::physics::traits::PhysicsState;
use crate::state::{SimState, SpatialOperations, SpatialState, StateError};
use nalgebra::{UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AersoPhysicsState {
    /// Spatial state containing position, velocity, attitude, and angular velocity
    pub spatial: SpatialState,
    /// Mass of the aircraft [kg]
    pub mass: f64,
    /// Force system tracking all forces and moments
    #[serde(skip)]
    pub forces: ForceSystem,
    /// Control inputs
    pub controls: AersoControls,
    /// Additional state variables specific to aerodynamic simulation
    pub aero: AeroState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AersoControls {
    pub throttle: f64, // [0, 1]
    pub elevator: f64, // [-1, 1]
    pub aileron: f64,  // [-1, 1]
    pub rudder: f64,   // [-1, 1]
    pub flaps: f64,    // [0, 1]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AeroState {
    pub angle_of_attack: f64,  // [rad]
    pub sideslip_angle: f64,   // [rad]
    pub dynamic_pressure: f64, // [Pa]
    pub air_density: f64,      // [kg/m³]
    pub air_speed: f64,        // [m/s]
    pub mach_number: f64,      // [-]
}

impl Default for AersoPhysicsState {
    fn default() -> Self {
        Self {
            spatial: SpatialState::default(),
            mass: 1.0, // Default mass in kg
            forces: ForceSystem::new(),
            controls: AersoControls::default(),
            aero: AeroState::default(),
        }
    }
}

impl Default for AersoControls {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            elevator: 0.0,
            aileron: 0.0,
            rudder: 0.0,
            flaps: 0.0,
        }
    }
}

impl Default for AeroState {
    fn default() -> Self {
        Self {
            angle_of_attack: 0.0,
            sideslip_angle: 0.0,
            dynamic_pressure: 0.0,
            air_density: 1.225, // Sea level standard density
            air_speed: 0.0,
            mach_number: 0.0,
        }
    }
}

impl SpatialOperations for AersoPhysicsState {
    fn position(&self) -> Vector3<f64> {
        self.spatial.position
    }

    fn velocity(&self) -> Vector3<f64> {
        self.spatial.velocity
    }

    fn attitude(&self) -> UnitQuaternion<f64> {
        self.spatial.attitude
    }

    fn angular_velocity(&self) -> Vector3<f64> {
        self.spatial.angular_velocity
    }

    fn set_position(&mut self, position: Vector3<f64>) -> Result<(), StateError> {
        self.spatial.position = position;
        Ok(())
    }

    fn set_velocity(&mut self, velocity: Vector3<f64>) -> Result<(), StateError> {
        self.spatial.velocity = velocity;
        Ok(())
    }

    fn set_attitude(&mut self, attitude: UnitQuaternion<f64>) -> Result<(), StateError> {
        self.spatial.attitude = attitude;
        Ok(())
    }

    fn set_angular_velocity(&mut self, angular_velocity: Vector3<f64>) -> Result<(), StateError> {
        self.spatial.angular_velocity = angular_velocity;
        Ok(())
    }
}

impl SimState for AersoPhysicsState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl PhysicsState for AersoPhysicsState {
    fn mass(&self) -> f64 {
        self.mass
    }

    fn add_force(&mut self, force: Vector3<f64>) {
        // Add force in body frame
        self.forces.add_force(Force::body_force(
            force,
            ForceType::Custom("External".into()),
            "external_force",
        ));
    }

    fn add_moment(&mut self, moment: Vector3<f64>) {
        // Add moment in body frame
        self.forces.add_moment(Moment::body_moment(
            moment,
            ForceType::Custom("External".into()),
            "external_moment",
        ));
    }

    fn clear_forces(&mut self) {
        self.forces.clear();
    }
}

impl AersoPhysicsState {
    pub fn new(mass: f64) -> Result<Self, StateError> {
        if mass <= 0.0 {
            return Err(StateError::InvalidValue("Mass must be positive".into()));
        }

        Ok(Self {
            spatial: SpatialState::default(),
            mass,
            forces: ForceSystem::new(),
            controls: AersoControls::default(),
            aero: AeroState::default(),
        })
    }

    pub fn reset(&mut self) {
        let mass = self.mass;
        *self = Self::default();
        self.mass = mass;
    }

    pub fn with_spatial(spatial: SpatialState, mass: f64) -> Result<Self, StateError> {
        if mass <= 0.0 {
            return Err(StateError::InvalidValue("Mass must be positive".into()));
        }

        Ok(Self {
            spatial,
            mass,
            forces: ForceSystem::new(),
            controls: AersoControls::default(),
            aero: AeroState::default(),
        })
    }

    /// Update aerodynamic state based on current conditions
    pub fn update_aero_state(&mut self, wind_velocity: Vector3<f64>) {
        let airspeed = self.spatial.velocity - wind_velocity;
        self.aero.air_speed = airspeed.norm();

        // Calculate angle of attack and sideslip
        if self.aero.air_speed > 1e-6 {
            let body_airspeed = self.spatial.attitude.inverse() * airspeed;
            self.aero.angle_of_attack = (body_airspeed.z / body_airspeed.x).atan();
            self.aero.sideslip_angle = (body_airspeed.y / self.aero.air_speed).asin();
        } else {
            self.aero.angle_of_attack = 0.0;
            self.aero.sideslip_angle = 0.0;
        }

        // Update dynamic pressure
        self.aero.dynamic_pressure = 0.5 * self.aero.air_density * self.aero.air_speed.powi(2);

        // Update Mach number (assuming standard sea level speed of sound = 340.29 m/s)
        self.aero.mach_number = self.aero.air_speed / 340.29;
    }

    /// Set control inputs with validation
    pub fn set_controls(&mut self, controls: AersoControls) -> Result<(), StateError> {
        // Validate control inputs
        if !(-1.0..=1.0).contains(&controls.elevator)
            || !(-1.0..=1.0).contains(&controls.aileron)
            || !(-1.0..=1.0).contains(&controls.rudder)
            || !(0.0..=1.0).contains(&controls.throttle)
            || !(0.0..=1.0).contains(&controls.flaps)
        {
            return Err(StateError::InvalidValue(
                "Control inputs out of range".into(),
            ));
        }

        self.controls = controls;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = AersoPhysicsState::new(10.0).unwrap();
        assert_eq!(state.mass(), 10.0);
    }

    #[test]
    fn test_invalid_mass() {
        assert!(AersoPhysicsState::new(-1.0).is_err());
    }

    #[test]
    fn test_aero_state_update() {
        let mut state = AersoPhysicsState::new(10.0).unwrap();
        state.spatial.velocity = Vector3::new(10.0, 0.0, 0.0);

        let wind = Vector3::zeros();
        state.update_aero_state(wind);

        assert!((state.aero.air_speed - 10.0).abs() < 1e-6);
        assert!(state.aero.angle_of_attack.abs() < 1e-6);
    }

    #[test]
    fn test_control_inputs() {
        let mut state = AersoPhysicsState::new(10.0).unwrap();

        let controls = AersoControls {
            throttle: 0.5,
            elevator: 0.0,
            aileron: 0.0,
            rudder: 0.0,
            flaps: 0.0,
        };

        assert!(state.set_controls(controls).is_ok());
    }
}
