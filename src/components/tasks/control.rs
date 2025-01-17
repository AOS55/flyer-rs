use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::components::{DubinsAircraftState, SpatialComponent, TaskComponent};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ControlParams {
    pub target: f64,
    pub tolerance: f64,
    pub control_type: ControlType,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ControlType {
    Altitude,
    Heading,
    Speed,
    Pitch,
    Roll,
}

impl Default for ControlType {
    fn default() -> Self {
        ControlType::Heading
    }
}

impl TaskComponent {
    pub fn calculate_dubins_control_reward(
        state: &DubinsAircraftState,
        params: &ControlParams,
    ) -> f64 {
        // Get the current value based on control type
        let current_value = match params.control_type {
            ControlType::Altitude => -state.spatial.position.z, // Convert from NED to altitude
            ControlType::Heading => {
                let mut heading = state.spatial.attitude.euler_angles().2;
                // Normalize heading to [0, 2π]
                while heading < 0.0 {
                    heading += 2.0 * PI;
                }
                while heading >= 2.0 * PI {
                    heading -= 2.0 * PI;
                }
                heading
            }
            ControlType::Speed => state.spatial.velocity.norm(),
            _ => {
                warn!(
                    "Control type {:?} not implemented for Dubins aircraft",
                    params.control_type
                );
                return 0.0;
            }
        };

        // Calculate error
        let mut error = (params.target - current_value).abs();

        // For heading, handle wraparound
        if params.control_type == ControlType::Heading {
            let direct_error = error;
            let wrapped_error = (2.0 * PI - error).abs();
            error = direct_error.min(wrapped_error);
        }

        // Calculate reward based on error and tolerance
        if error <= params.tolerance {
            // Maximum reward when within tolerance
            1.0
        } else {
            // Exponential decay based on error
            let scale_factor = 5.0; // Controls how quickly reward drops off
            (-scale_factor * error / params.tolerance).exp()
        }
    }

    pub fn calculate_full_control_reward(
        spatial: &SpatialComponent,
        params: &ControlParams,
    ) -> f64 {
        // Get the current value based on control type
        let current_value = match params.control_type {
            ControlType::Altitude => -spatial.position.z, // Convert from NED to altitude
            ControlType::Heading => {
                let mut heading = spatial.attitude.euler_angles().2;
                // Normalize heading to [0, 2π]
                while heading < 0.0 {
                    heading += 2.0 * PI;
                }
                while heading >= 2.0 * PI {
                    heading -= 2.0 * PI;
                }
                heading
            }
            ControlType::Speed => spatial.velocity.norm(),
            ControlType::Pitch => spatial.attitude.euler_angles().1,
            ControlType::Roll => spatial.attitude.euler_angles().0,
        };

        // Calculate error
        let mut error = (params.target - current_value).abs();

        // For heading and yaw, handle wraparound
        if params.control_type == ControlType::Heading {
            let direct_error = error;
            let wrapped_error = (2.0 * PI - error).abs();
            error = direct_error.min(wrapped_error);
        }

        // Calculate reward based on error and tolerance
        if error <= params.tolerance {
            // Maximum reward when within tolerance
            1.0
        } else {
            // Exponential decay based on error
            let scale_factor = 5.0; // Controls how quickly reward drops off
            (-scale_factor * error / params.tolerance).exp()
        }
    }
}
