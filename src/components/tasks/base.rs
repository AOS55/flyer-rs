use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{
    collision::CollisionComponent,
    tasks::{
        control::ControlParams, goal::GoalParams, landing::LandingParams, runway::RunwayParams,
        trajectory::TrajectoryParams,
    },
    DubinsAircraftState, SpatialComponent,
};

#[derive(Component, Deserialize, Serialize, Debug, Clone)]
pub struct TaskComponent {
    pub task_type: TaskType,
    pub terminated: bool,
    pub weight: f64,
}

impl Default for TaskComponent {
    fn default() -> Self {
        Self {
            task_type: Default::default(),
            terminated: true,
            weight: 1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TaskType {
    /// Low-level control task with target
    Control(ControlParams),
    /// Navigate to 3D goal position
    Goal(GoalParams),
    /// Follow defined trajectory
    Trajectory(TrajectoryParams),
    /// Navigate to runway
    Runway(RunwayParams),
    /// Execute forced landing
    ForcedLanding(LandingParams),
}

impl Default for TaskType {
    fn default() -> Self {
        TaskType::Control(ControlParams {
            target: 0.0,
            tolerance: 2.0,
            control_type: Default::default(),
        })
    }
}

impl TaskComponent {
    /// Calculate reward for Dubins Aircraft
    pub fn calculate_dubins_reward(
        &self,
        state: &DubinsAircraftState,
        collision: &CollisionComponent,
    ) -> f64 {
        match &self.task_type {
            TaskType::Control(params) => {
                TaskComponent::calculate_dubins_control_reward(state, &params)
            }
            TaskType::Goal(params) => TaskComponent::calculate_goal_reward(&state.spatial, &params),
            TaskType::Trajectory(params) => {
                TaskComponent::calculate_trajectory_reward(&state.spatial, &params)
            }
            TaskType::Runway(params) => {
                TaskComponent::calculate_runway_reward(&state.spatial, &params)
            }
            TaskType::ForcedLanding(params) => {
                TaskComponent::calculate_dubins_forced_landing_reward(
                    &state.spatial,
                    collision,
                    &params,
                )
            }
        }
    }

    /// Calculate reward for Full Aircraft
    pub fn calculate_full_reward(
        &self,
        spatial: &SpatialComponent,
        collision: &CollisionComponent,
    ) -> f64 {
        match &self.task_type {
            TaskType::Control(params) => {
                TaskComponent::calculate_full_control_reward(spatial, &params)
            }
            TaskType::Goal(params) => TaskComponent::calculate_goal_reward(spatial, &params),
            TaskType::Trajectory(params) => {
                TaskComponent::calculate_trajectory_reward(spatial, &params)
            }
            TaskType::Runway(params) => TaskComponent::calculate_runway_reward(spatial, &params),
            TaskType::ForcedLanding(params) => {
                TaskComponent::calculate_full_forced_landing_reward(spatial, collision, &params)
            }
        }
    }

    /// Determine if task is terminated
    pub fn is_dubins_terminated(&self, state: &DubinsAircraftState) -> bool {
        match &self.task_type {
            TaskType::Control(_params) => TaskComponent::simple_termination(&state.spatial),
            TaskType::Goal(params) => TaskComponent::goal_termination(&state.spatial, &params),
            TaskType::Trajectory(params) => {
                TaskComponent::trajectory_termination(&state.spatial, &params)
            }
            TaskType::Runway(_params) => TaskComponent::simple_termination(&state.spatial), // TODO: Implement Runway termination based on params
            TaskType::ForcedLanding(_params) => TaskComponent::simple_termination(&state.spatial), // TODO: Implement ForcedLanding termination based on params
        }
    }
}
