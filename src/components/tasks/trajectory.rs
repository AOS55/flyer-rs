use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::components::{SpatialComponent, TaskComponent};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrajectoryParams {
    pub motion_type: TrajectoryMotionPrimitive,
    pub target_velocity: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
pub enum TrajectoryMotionPrimitive {
    // Straight and level flight
    StraightAndLevel {
        target_distance: f64,
    },
    // Straight line followed by turn
    StraightAndTurn {
        straight_distance: f64,
        turn_radius: f64,
        turn_angle: f64,
        direction: TurnDirection,
    },
    // Climbing motion
    Climb {
        target_climb_rate: f64,
    },
    // Descending motion
    Descend {
        target_descent_rate: f64,
    },
    // Coordinated turn (constant radius turn)
    CoordinatedTurn {
        turn_radius: f64,
        turn_angle: f64,
        direction: TurnDirection,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum TurnDirection {
    Left,
    Right,
}

impl TaskComponent {
    pub fn calculate_trajectory_reward(state: &SpatialComponent, params: &TrajectoryParams) -> f64 {
        let velocity_reward = Self::calculate_velocity_reward(state, params.target_velocity);
        let motion_reward = Self::calculate_motion_primitive_reward(state, params);

        // Combine rewards
        0.4 * velocity_reward + 0.6 * motion_reward
    }

    fn calculate_velocity_reward(state: &SpatialComponent, target_velocity: f64) -> f64 {
        let velocity_error = (state.velocity.norm() - target_velocity).abs();
        let velocity_tolerance = 2.0; // m/s

        if velocity_error <= velocity_tolerance {
            1.0
        } else {
            (-5.0 * velocity_error / velocity_tolerance).exp()
        }
    }

    fn calculate_motion_primitive_reward(
        state: &SpatialComponent,
        params: &TrajectoryParams,
    ) -> f64 {
        match &params.motion_type {
            #[allow(unused_variables)]
            TrajectoryMotionPrimitive::StraightAndLevel { target_distance } => {
                let (roll, pitch, _) = state.attitude.euler_angles();

                // Penalize deviations from level flight
                let attitude_error = roll.abs() + pitch.abs();
                let attitude_reward = (-3.0 * attitude_error / PI).exp();

                // Penalize angular rates
                let angular_rate_error = state.angular_velocity.norm();
                let rate_reward = (-2.0 * angular_rate_error).exp();

                // Could add distance tracking reward here if needed
                0.6 * attitude_reward + 0.4 * rate_reward
            }
            #[allow(unused_variables)]
            TrajectoryMotionPrimitive::CoordinatedTurn {
                turn_radius,
                turn_angle,
                direction,
            } => {
                let velocity = state.velocity.norm();
                let (roll, pitch, _) = state.attitude.euler_angles();

                // Calculate expected bank angle for coordinated turn
                let expected_bank = if velocity < 1.0 {
                    0.0 // Prevent division by zero at very low speeds
                } else {
                    (velocity.powi(2) / (9.81 * turn_radius)).atan()
                };
                let expected_bank = match direction {
                    TurnDirection::Left => -expected_bank,
                    TurnDirection::Right => expected_bank,
                };

                // Bank angle error
                let bank_error = (roll - expected_bank).abs();
                let bank_reward = (-3.0 * bank_error / PI).exp();

                // Pitch should remain close to zero
                let pitch_reward = (-3.0 * pitch.abs() / PI).exp();

                // Turn rate should match expected
                let expected_turn_rate = if velocity < 1.0 {
                    0.0
                } else {
                    velocity / turn_radius
                };
                let turn_rate_error = (state.angular_velocity.z - expected_turn_rate).abs();
                let turn_rate_reward = (-2.0 * turn_rate_error).exp();

                0.4 * bank_reward + 0.3 * pitch_reward + 0.3 * turn_rate_reward
            }
            TrajectoryMotionPrimitive::Climb { target_climb_rate } => {
                let (roll, pitch, _) = state.attitude.euler_angles();
                let vertical_speed = -state.velocity.z; // NED to altitude rate
                let horizontal_speed = state.velocity.xy().norm();

                // Vertical speed error
                let rate_error = (vertical_speed - target_climb_rate).abs();
                let rate_reward = (-2.0 * rate_error).exp();

                // Should maintain wings level
                let roll_reward = (-3.0 * roll.abs() / PI).exp();

                // Pitch should be consistent with climb rate
                let expected_pitch = if horizontal_speed < 1.0 {
                    0.0 // Prevent extreme pitch at very low speeds
                } else {
                    (target_climb_rate / horizontal_speed).atan()
                };
                let pitch_error = (pitch - expected_pitch).abs();
                let pitch_reward = (-3.0 * pitch_error / PI).exp();

                0.4 * rate_reward + 0.3 * roll_reward + 0.3 * pitch_reward
            }
            TrajectoryMotionPrimitive::Descend {
                target_descent_rate,
            } => {
                let (roll, pitch, _) = state.attitude.euler_angles();
                let vertical_speed = -state.velocity.z; // NED to altitude rate
                let horizontal_speed = state.velocity.xy().norm();

                // Vertical speed error
                let rate_error = (vertical_speed + target_descent_rate).abs();
                let rate_reward = (-2.0 * rate_error).exp();

                // Should maintain wings level
                let roll_reward = (-3.0 * roll.abs() / PI).exp();

                // Pitch should be consistent with descent rate
                let expected_pitch = if horizontal_speed < 1.0 {
                    0.0 // Prevent extreme pitch at very low speeds
                } else {
                    (-target_descent_rate / horizontal_speed).atan()
                };
                let pitch_error = (pitch - expected_pitch).abs();
                let pitch_reward = (-3.0 * pitch_error / PI).exp();

                0.4 * rate_reward + 0.3 * roll_reward + 0.3 * pitch_reward
            }
            TrajectoryMotionPrimitive::StraightAndTurn {
                straight_distance,
                turn_radius,
                turn_angle,
                direction,
            } => {
                let straight_reward = Self::calculate_motion_primitive_reward(
                    state,
                    &TrajectoryParams {
                        motion_type: TrajectoryMotionPrimitive::StraightAndLevel {
                            target_distance: *straight_distance,
                        },
                        target_velocity: params.target_velocity,
                    },
                );

                let turn_reward = Self::calculate_motion_primitive_reward(
                    state,
                    &TrajectoryParams {
                        motion_type: TrajectoryMotionPrimitive::CoordinatedTurn {
                            turn_radius: *turn_radius,
                            turn_angle: *turn_angle,
                            direction: *direction,
                        },
                        target_velocity: params.target_velocity,
                    },
                );

                0.5 * (straight_reward + turn_reward)
            }
        }
    }

    pub fn trajectory_termination(state: &SpatialComponent, params: &TrajectoryParams) -> bool {
        match params.motion_type {
            TrajectoryMotionPrimitive::StraightAndLevel { target_distance } => {
                // Check if the traveled distance exceeds or equals the target distance
                state.position.xy().norm() >= target_distance
            }
            #[allow(unused_variables)]
            TrajectoryMotionPrimitive::CoordinatedTurn {
                turn_radius,
                turn_angle,
                direction: _,
            } => {
                // Compute the completed turn angle based on angular position or rate
                let completed_angle = state.attitude.euler_angles().2.abs(); // Assuming yaw gives turn angle
                completed_angle >= turn_angle
            }
            TrajectoryMotionPrimitive::Climb { target_climb_rate } => {
                // Check if vertical velocity has reached or surpassed the target climb rate
                let vertical_speed = -state.velocity.z; // Assuming NED frame
                (vertical_speed - target_climb_rate).abs() < 0.1 // Allow small tolerance
            }
            TrajectoryMotionPrimitive::Descend {
                target_descent_rate,
            } => {
                // Check if vertical velocity has reached or surpassed the target descent rate
                let vertical_speed = -state.velocity.z; // Assuming NED frame
                (vertical_speed + target_descent_rate).abs() < 0.1 // Allow small tolerance
            }
            TrajectoryMotionPrimitive::StraightAndTurn {
                straight_distance,
                turn_radius,
                turn_angle,
                direction,
            } => {
                // Check if straight and turn components have been completed
                let straight_terminated = Self::trajectory_termination(
                    state,
                    &TrajectoryParams {
                        motion_type: TrajectoryMotionPrimitive::StraightAndLevel {
                            target_distance: straight_distance,
                        },
                        target_velocity: params.target_velocity,
                    },
                );
                let turn_terminated = Self::trajectory_termination(
                    state,
                    &TrajectoryParams {
                        motion_type: TrajectoryMotionPrimitive::CoordinatedTurn {
                            turn_radius,
                            turn_angle,
                            direction,
                        },
                        target_velocity: params.target_velocity,
                    },
                );
                straight_terminated && turn_terminated
            }
        }
    }
}
