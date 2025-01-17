use crate::components::{tasks::GoalParams, SpatialComponent, TaskComponent};

impl TaskComponent {
    pub fn simple_termination(state: &SpatialComponent) -> bool {
        state.position.z < 0.0
    }

    pub fn goal_termination(state: &SpatialComponent, params: &GoalParams) -> bool {
        let distance = (params.position - state.position).norm();
        distance < params.tolerance
    }
}
