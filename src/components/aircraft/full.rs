use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{
    AirData, FullAircraftConfig, PhysicsComponent, PropulsionState, SpatialComponent, StartConfig,
};

/// Represents the overall state of an aircraft (just a convenience method NOT a component)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullAircraftState {
    /// Aerodynamic data of the aircraft (e.g., airspeed, angle of attack).
    pub air_data: AirData,
    /// Positions of control surfaces like elevator, aileron, rudder, and flaps.
    pub control_surfaces: AircraftControlSurfaces,
    /// Spatial data such as position, orientation, and velocity.
    pub spatial: SpatialComponent,
    /// Physical properties like mass and inertia.
    pub physics: PhysicsComponent,
    /// State of the propulsion system
    pub propulsion: PropulsionState,
}

// impl Default for FullAircraftState {
//     /// Provides a default state for the aircraft.
//     /// Default values are based on the Twin Otter aircraft configuration.
//     fn default() -> Self {
//         Self {
//             air_data: AirData::default(),
//             control_surfaces: AircraftControlSurfaces::default(),
//             spatial: SpatialComponent::default(),
//             physics: PhysicsComponent::new(
//                 4874.8,
//                 Matrix3::from_columns(&[
//                     Vector3::new(28366.4, 0.0, -1384.3),
//                     Vector3::new(0.0, 32852.8, 0.0),
//                     Vector3::new(-1384.3, 0.0, 52097.3),
//                 ]),
//             ),
//             propulsion: PowerplantState::default(),
//         }
//     }
// }

impl FullAircraftState {
    /// Creates a new `FullAircraftState` from a given configuration.
    pub fn from_config(config: &FullAircraftConfig) -> Self {
        let (position, speed, heading) = match config.start_config {
            StartConfig::Fixed(fixed_config) => {
                let position = fixed_config.position;
                let speed = fixed_config.speed;
                let heading = fixed_config.heading;
                (position, speed, heading)
            }
            StartConfig::Random(random_config) => random_config.generate(),
        };

        Self {
            air_data: AirData::default(), // Will update on first step of state
            control_surfaces: AircraftControlSurfaces::default(),
            spatial: SpatialComponent::at_position_and_airspeed(position, speed, heading),
            physics: PhysicsComponent::new(config.mass.mass, config.mass.inertia),
            propulsion: PropulsionState::default(),
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
    /// Power level position (0-1).
    pub power_lever: f64,
}

impl Default for AircraftControlSurfaces {
    /// Provides a default state where all control surfaces are neutral (0.0).
    fn default() -> Self {
        Self {
            elevator: 0.0,
            aileron: 0.0,
            rudder: 0.0,
            power_lever: 0.5,
        }
    }
}
