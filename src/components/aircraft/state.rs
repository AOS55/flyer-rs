use bevy::prelude::*;
use nalgebra::{Matrix3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

use crate::components::{PhysicsComponent, RandomStartConfig, SpatialComponent};

/// Represents the overall state of an aircraft.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct FullAircraftState {
    /// Aerodynamic data of the aircraft (e.g., airspeed, angle of attack).
    pub air_data: AirData,
    /// Positions of control surfaces like elevator, aileron, rudder, and flaps.
    pub control_surfaces: AircraftControlSurfaces,
    /// Spatial data such as position, orientation, and velocity.
    pub spatial: SpatialComponent,
    /// Physical properties like mass and inertia.
    pub physics: PhysicsComponent,
}

impl Default for FullAircraftState {
    /// Provides a default state for the aircraft.
    /// Default values are based on the Twin Otter aircraft configuration.
    fn default() -> Self {
        Self {
            air_data: AirData::default(),
            control_surfaces: AircraftControlSurfaces::default(),
            spatial: SpatialComponent::default(),
            physics: PhysicsComponent::new(
                4874.8,
                Matrix3::from_columns(&[
                    Vector3::new(28366.4, 0.0, -1384.3),
                    Vector3::new(0.0, 32852.8, 0.0),
                    Vector3::new(-1384.3, 0.0, 52097.3),
                ]),
            ),
        }
    }
}

/// Represents the positions of the aircraft's control surfaces.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Copy)]
pub struct AircraftControlSurfaces {
    /// Elevator deflection (radians).
    pub elevator: f64,
    /// Aileron deflection (radians).
    pub aileron: f64,
    /// Rudder deflection (radians).
    pub rudder: f64,
    /// Flap deflection (radians).
    pub flaps: f64,
}

impl Default for AircraftControlSurfaces {
    /// Provides a default state where all control surfaces are neutral (0.0).
    fn default() -> Self {
        Self {
            elevator: 0.0,
            aileron: 0.0,
            rudder: 0.0,
            flaps: 0.0,
        }
    }
}

/// Represents aerodynamic data for the aircraft.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AirData {
    /// True airspeed of the aircraft (m/s).
    pub true_airspeed: f64,
    /// Angle of attack (α) in radians.
    pub alpha: f64,
    /// Sideslip angle (β) in radians.
    pub beta: f64,
    /// Dynamic pressure acting on the aircraft (Pa).
    pub dynamic_pressure: f64,
    /// Air density (kg/m³).
    pub density: f64,
    /// Relative velocity vector of the aircraft (m/s).
    pub relative_velocity: Vector3<f64>,
    /// Wind velocity vector (m/s).
    pub wind_velocity: Vector3<f64>,
}

impl Default for AirData {
    /// Provides a default state where all aerodynamic values are zero.
    fn default() -> Self {
        Self {
            true_airspeed: 0.0,
            alpha: 0.0,
            beta: 0.0,
            dynamic_pressure: 0.0,
            density: 0.0,
            relative_velocity: Vector3::zeros(),
            wind_velocity: Vector3::zeros(),
        }
    }
}

/// Represents the state of a Dubins aircraft.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DubinsAircraftState {
    /// Spatial data such as position and orientation.
    pub spatial: SpatialComponent,
    /// Control inputs for acceleration, bank angle, and vertical speed.
    pub controls: DubinsAircraftControls,
}

impl Default for DubinsAircraftState {
    /// Provides a default Dubins aircraft state with neutral controls and zeroed spatial data.
    fn default() -> Self {
        Self {
            spatial: SpatialComponent::default(),
            controls: DubinsAircraftControls::default(),
        }
    }
}

impl DubinsAircraftState {
    /// Creates a new default Dubins aircraft state.
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: Make a single system that takes config and Precise/Random start are on an enum

    /// Starts the aircraft at an exact position specified in NED coordinates.
    ///
    /// # Arguments
    /// * `position` - A `Vector3` representing the position in NED coordinates.
    pub fn precise_start(position: Vector3<f64>) -> Self {
        let spatial = SpatialComponent::at_position(position);
        let controls = DubinsAircraftControls::default();

        Self { spatial, controls }
    }

    /// Starts the aircraft at a random position on a hemisphere.
    ///
    /// # Arguments
    /// * `config` - Optional configuration for randomized start positions.
    ///
    /// # Returns
    /// A `DubinsAircraftState` with a random position and neutral controls.
    pub fn random_start(config: Option<RandomStartConfig>) -> Self {
        let config = config.unwrap_or_default();
        let (position, speed, heading) = config.generate();
        let mut spatial = SpatialComponent::at_position(position);
        spatial.velocity = Vector3::new(
            speed * heading.cos(),
            speed * heading.sin(),
            0.0, // Initial vertical velocity set to 0
        );
        spatial.attitude = UnitQuaternion::from_euler_angles(0.0, 0.0, heading);

        Self {
            spatial,
            controls: DubinsAircraftControls::default(),
        }
    }
}

/// Represents the simplified control inputs for a Dubins aircraft.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DubinsAircraftControls {
    /// Acceleration of the aircraft (m/s²).
    pub acceleration: f64,
    /// Bank angle (radians), determines turning behavior.
    pub bank_angle: f64,
    /// Vertical speed of the aircraft (m/s).
    pub vertical_speed: f64,
}

impl Default for DubinsAircraftControls {
    /// Provides a default state with all control inputs set to zero.
    fn default() -> Self {
        Self {
            acceleration: 0.0,
            bank_angle: 0.0,
            vertical_speed: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AircraftControls {
    Dubins(DubinsAircraftControls),
    Full(AircraftControlSurfaces),
}

#[derive(Debug, Clone)]
pub enum AircraftState {
    Dubins(DubinsAircraftState),
    Full(FullAircraftState),
}
