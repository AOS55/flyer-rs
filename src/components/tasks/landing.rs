use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::components::{CollisionComponent, SpatialComponent, TaskComponent};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LandingParams {
    /// Target landing position (for precision landing)
    pub target_position: Option<Vector3<f64>>,
    /// Maximum acceptable landing speed (m/s)
    pub max_landing_speed: f64,
    /// Maximum acceptable descent rate (m/s)
    pub max_descent_rate: f64,
    /// Maximum acceptable bank angle during landing (radians)
    pub max_bank_angle: f64,
    /// Max distance to runway or landing zone for precision landings
    pub max_landing_distance: f64,
    /// Landing complete threshold (height above ground)
    pub landing_complete_height: f64,
}

impl Default for LandingParams {
    fn default() -> Self {
        Self {
            target_position: None,
            max_landing_speed: 25.0,      // m/s
            max_descent_rate: 3.0,        // m/s
            max_bank_angle: PI / 6.0,     // 30 degrees
            max_landing_distance: 200.0,  // meters
            landing_complete_height: 0.5, // meters
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LandingResult {
    Success,
    TooFast,
    HighDescentRate,
    BadAttitude,
    WrongLocation,
}

impl TaskComponent {
    /// Calculate landing reward for Dubins aircraft
    pub fn calculate_dubins_forced_landing_reward(
        spatial: &SpatialComponent,
        collision: &CollisionComponent,
        params: &LandingParams,
    ) -> f64 {
        // Only give reward at termination
        if !collision.has_collided {
            return 0.0;
        }

        // Get landing result
        let landing_result = evaluate_landing(spatial, params);

        // Calculate reward based on landing result
        match landing_result {
            LandingResult::Success => 1.0,
            LandingResult::TooFast => -0.5,
            LandingResult::HighDescentRate => -0.5,
            LandingResult::BadAttitude => -0.5,
            LandingResult::WrongLocation => -0.25,
        }
    }

    /// Calculate landing reward for full aircraft
    pub fn calculate_full_forced_landing_reward(
        spatial: &SpatialComponent,
        collision: &CollisionComponent,
        params: &LandingParams,
    ) -> f64 {
        // Only give reward at termination
        if !collision.has_collided {
            return 0.0;
        }

        // Get landing result
        let landing_result = evaluate_landing(spatial, params);

        // Calculate reward based on landing result
        match landing_result {
            LandingResult::Success => 1.0,
            LandingResult::TooFast => -0.5,
            LandingResult::HighDescentRate => -0.5,
            LandingResult::BadAttitude => -0.5,
            LandingResult::WrongLocation => -0.25,
        }
    }

    pub fn landing_termination(
        state: &SpatialComponent,
        collision: &CollisionComponent,
        params: &LandingParams,
    ) -> bool {
        // Check for ground contact or collision
        let height = -state.position.z;
        if height < params.landing_complete_height || collision.has_collided {
            return true;
        }

        // Get ground speed and descent rate
        let ground_speed = state.velocity.xy().norm();
        let descent_rate = -state.velocity.z;
        let (bank, _, _) = state.attitude.euler_angles();

        // Terminate for unsafe conditions
        if ground_speed > params.max_landing_speed * 1.5
            || descent_rate > params.max_descent_rate * 1.5
            || bank.abs() > params.max_bank_angle * 1.5
        {
            return true;
        }

        // For precision landings, terminate if too far from target
        if let Some(target) = params.target_position {
            let distance = (target - state.position).xy().norm();
            if distance > params.max_landing_distance {
                return true;
            }
        }

        false
    }
}

fn evaluate_landing(state: &SpatialComponent, params: &LandingParams) -> LandingResult {
    // Get important state values
    let ground_speed = state.velocity.xy().norm();
    let descent_rate = -state.velocity.z;
    let (bank, pitch, _) = state.attitude.euler_angles();

    // Check speed
    if ground_speed > params.max_landing_speed {
        return LandingResult::TooFast;
    }

    // Check descent rate
    if descent_rate > params.max_descent_rate {
        return LandingResult::HighDescentRate;
    }

    // Check attitude
    if bank.abs() > params.max_bank_angle || pitch.abs() > params.max_bank_angle {
        return LandingResult::BadAttitude;
    }

    // For precision landing, check position
    if let Some(target) = params.target_position {
        let distance = (target - state.position).xy().norm();
        if distance > params.max_landing_distance {
            return LandingResult::WrongLocation;
        }
    }

    LandingResult::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::UnitQuaternion;

    fn create_test_spatial(
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        bank: f64,
        pitch: f64,
    ) -> SpatialComponent {
        SpatialComponent {
            position,
            velocity,
            attitude: UnitQuaternion::from_euler_angles(bank, pitch, 0.0),
            angular_velocity: Vector3::zeros(),
        }
    }

    #[test]
    fn test_landing_success() {
        let params = LandingParams::default();
        let spatial = create_test_spatial(
            Vector3::new(0.0, 0.0, -0.1),  // Near ground
            Vector3::new(10.0, 0.0, -1.0), // Safe approach speed
            0.0,                           // Level bank
            0.0,                           // Level pitch
        );

        let result = evaluate_landing(&spatial, &params);
        assert!(matches!(result, LandingResult::Success));
    }

    #[test]
    fn test_landing_too_fast() {
        let params = LandingParams::default();
        let spatial = create_test_spatial(
            Vector3::new(0.0, 0.0, -0.1),
            Vector3::new(50.0, 0.0, -1.0), // Too fast
            0.0,
            0.0,
        );

        let result = evaluate_landing(&spatial, &params);
        assert!(matches!(result, LandingResult::TooFast));
    }

    #[test]
    fn test_landing_high_descent() {
        let params = LandingParams::default();
        let spatial = create_test_spatial(
            Vector3::new(0.0, 0.0, -0.1),
            Vector3::new(10.0, 0.0, -10.0), // High descent rate
            0.0,
            0.0,
        );

        let result = evaluate_landing(&spatial, &params);
        assert!(matches!(result, LandingResult::HighDescentRate));
    }

    #[test]
    fn test_landing_bad_attitude() {
        let params = LandingParams::default();
        let spatial = create_test_spatial(
            Vector3::new(0.0, 0.0, -0.1),
            Vector3::new(10.0, 0.0, -1.0),
            PI / 3.0, // Excessive bank
            0.0,
        );

        let result = evaluate_landing(&spatial, &params);
        assert!(matches!(result, LandingResult::BadAttitude));
    }

    #[test]
    fn test_landing_precision() {
        let mut params = LandingParams::default();
        params.target_position = Some(Vector3::new(0.0, 0.0, 0.0));

        // Test landing far from target
        let spatial = create_test_spatial(
            Vector3::new(300.0, 0.0, -0.1), // Too far from target
            Vector3::new(10.0, 0.0, -1.0),
            0.0,
            0.0,
        );

        let result = evaluate_landing(&spatial, &params);
        assert!(matches!(result, LandingResult::WrongLocation));
    }
}
