use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{SpatialComponent, StartConfig};

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
    /// Creates a new Dubins aircraft state from a given configuration.
    pub fn from_config(config: &StartConfig) -> Self {
        match config {
            StartConfig::Fixed(fixed_config) => {
                let spatial = SpatialComponent::at_position_and_airspeed(
                    fixed_config.position,
                    fixed_config.speed,
                    fixed_config.heading,
                );
                Self {
                    spatial,
                    controls: DubinsAircraftControls::default(),
                }
            }
            StartConfig::Random(random_config) => {
                let (position, speed, heading) = random_config.generate();
                let spatial = SpatialComponent::at_position_and_airspeed(position, speed, heading);
                Self {
                    spatial,
                    controls: DubinsAircraftControls::default(),
                }
            }
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
