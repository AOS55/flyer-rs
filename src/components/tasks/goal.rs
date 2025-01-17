use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use crate::components::{SpatialComponent, TaskComponent};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GoalParams {
    pub position: Vector3<f64>,
    pub reward_type: GoalRewardType,
    pub tolerance: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GoalRewardType {
    Sparse,
    Dense,
}

impl TaskComponent {
    pub fn calculate_goal_reward(state: &SpatialComponent, params: &GoalParams) -> f64 {
        // Calculate distance to goal
        let distance = (params.position - state.position).norm();

        match params.reward_type {
            GoalRewardType::Sparse => {
                // Binary reward: 1.0 if within tolerance, 0.0 otherwise
                if distance <= params.tolerance {
                    1.0
                } else {
                    0.0
                }
            }
            GoalRewardType::Dense => {
                // Exponential decay based on distance
                let scale_factor = 5.0; // Controls how quickly reward drops off
                if distance <= params.tolerance {
                    1.0
                } else {
                    // Normalize distance by tolerance and apply exponential decay
                    (-scale_factor * distance / params.tolerance).exp()
                }
            }
        }
    }
}
