use nalgebra::{Matrix3, Rotation3, Vector3};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::components::{SpatialComponent, TaskComponent};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RunwayParams {
    pub position: Vector3<f64>,
    pub heading: f64,
    pub width: f64,
    pub length: f64,
    pub glideslope: f64,
}

impl TaskComponent {
    pub fn calculate_runway_reward(state: &SpatialComponent, params: &RunwayParams) -> f64 {
        // Create runway coordinate system (x along runway, y perpendicular, z up)
        let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), params.heading);
        let runway_to_ned = Matrix3::from(rotation);
        let ned_to_runway = runway_to_ned.transpose();

        // Convert aircraft position to runway coordinates
        let relative_position = state.position - params.position;
        let position_runway = ned_to_runway * relative_position;

        // Calculate lateral deviation (cross-track error)
        let lateral_error = position_runway.y.abs();
        let lateral_tolerance = params.width;
        let lateral_reward = if lateral_error <= lateral_tolerance {
            1.0
        } else {
            (-3.0 * lateral_error / lateral_tolerance).exp()
        };

        // Calculate glideslope error
        let distance_to_threshold = position_runway.x;
        let desired_height = distance_to_threshold * params.glideslope.tan();
        let height_error = (position_runway.z - desired_height).abs();
        let height_tolerance = 10.0; // meters
        let glideslope_reward = (-2.0 * height_error / height_tolerance).exp();

        // Calculate heading alignment
        let aircraft_heading = state.attitude.euler_angles().2;
        let heading_error = (aircraft_heading - params.heading).abs();
        let heading_error = heading_error.min((2.0 * PI - heading_error).abs()); // Handle wraparound
        let heading_tolerance = 15.0 * PI / 180.0; // 15 degrees in radians
        let heading_reward = (-3.0 * heading_error / heading_tolerance).exp();

        // Calculate approach speed
        let ground_speed = state.velocity.xy().norm();
        let target_approach_speed = 30.0; // m/s, adjust as needed
        let speed_error = (ground_speed - target_approach_speed).abs();
        let speed_tolerance = 5.0; // m/s
        let speed_reward = (-2.0 * speed_error / speed_tolerance).exp();

        // Combine rewards with weights
        0.3 * lateral_reward + 0.3 * glideslope_reward + 0.2 * heading_reward + 0.2 * speed_reward
    }

    pub fn runway_termination(state: &SpatialComponent, params: &RunwayParams) -> bool {
        // Create runway coordinate system
        let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), params.heading);
        let runway_to_ned = Matrix3::from(rotation);
        let ned_to_runway = runway_to_ned.transpose();

        // Convert aircraft position to runway coordinates
        let relative_position = state.position - params.position;
        let position_runway = ned_to_runway * relative_position;

        // Check if aircraft has crossed runway threshold
        if position_runway.x < 0.0 || position_runway.x > params.length {
            return true;
        }

        // Check if aircraft has touched down
        if -state.position.z < 0.5 {
            // Height above ground less than 0.5m
            return true;
        }

        // Check if aircraft has deviated too far laterally
        if position_runway.y.abs() > params.width * 2.0 {
            return true;
        }

        false
    }
}
