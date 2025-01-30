use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::components::{DubinsAircraftState, SpatialComponent, TaskComponent};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ControlParams {
    pub target: f64,
    pub tolerance: f64,
    pub control_type: ControlType,
    pub prev_value: Option<f64>,
}

impl Default for ControlParams {
    fn default() -> Self {
        Self {
            target: 0.0,
            tolerance: 1.0,
            control_type: ControlType::default(),
            prev_value: None,
        }
    }
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
        params: &mut ControlParams,
    ) -> f64 {
        let current_value = match params.control_type {
            ControlType::Altitude => -state.spatial.position.z,
            ControlType::Heading => {
                let mut heading = state.spatial.attitude.euler_angles().2;
                while heading < 0.0 {
                    heading += 2.0 * PI;
                }
                while heading >= 2.0 * PI {
                    heading -= 2.0 * PI;
                }
                heading
            }
            ControlType::Speed => state.spatial.velocity.norm(),
            ControlType::Pitch => state.spatial.attitude.euler_angles().1,
            ControlType::Roll => state.spatial.attitude.euler_angles().0,
        };

        let mut error = (params.target - current_value).abs();

        // Handle heading wraparound
        if params.control_type == ControlType::Heading {
            let direct_error = error;
            let wrapped_error = (2.0 * PI - error).abs();
            error = direct_error.min(wrapped_error);
        }

        // Normalize error relative to tolerance, with adjusted scaling
        let normalized_error = error / (3.0 * params.tolerance); // Reduced from 5.0 to 3.0

        // Base reward with steeper curve and higher maximum
        let base_reward = (-normalized_error * normalized_error).exp(); // Squared term for sharper peak

        // Progress reward that's always positive during descent
        let progress_reward = if let Some(prev_value) = params.prev_value {
            let prev_error = (params.target - prev_value).abs();
            let error_improvement = prev_error - error;

            // Scale improvement more aggressively
            let progress_scale = match params.control_type {
                ControlType::Altitude => 2.0, // Larger scale for altitude
                ControlType::Heading => 1.5,  // Moderate scale for heading
                _ => 1.0,                     // Default scale for others
            };

            progress_scale * (1.0 + f64::tanh(2.0 * error_improvement))
        } else {
            0.0
        };

        // Smoother stability term
        let stability_term = if error > 3.0 * params.tolerance {
            let excess = error - 3.0 * params.tolerance;
            -0.2 * (excess / params.tolerance).tanh() // Gentler penalty
        } else {
            0.0
        };

        // Update previous value
        params.prev_value = Some(current_value);

        // Combine with adjusted weights and scale to [0,1]
        let raw_reward = 0.7 * base_reward +       // Increased weight on target proximity
                0.25 * progress_reward +  // Significant weight on progress
                0.05 * stability_term; // Minor stability influence

        // Final scaling to ensure [0,1] range
        (raw_reward.max(0.0)).min(1.0)
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
