use bevy::prelude::*;

use crate::{
    components::{tasks::TaskComponent, CollisionComponent, DubinsAircraftState, SpatialComponent},
    plugins::Identifier,
    resources::AgentState,
};

pub fn determine_terminated(
    dubins_query: Query<(
        &Identifier,
        &DubinsAircraftState,
        &TaskComponent,
        &CollisionComponent,
    )>,
    full_query: Query<(
        &Identifier,
        &SpatialComponent,
        &TaskComponent,
        &CollisionComponent,
    )>,
    agent_state: ResMut<AgentState>,
) {
    if let Ok(mut termination_buffer) = agent_state.termination_buffer.lock() {
        termination_buffer.clear();

        for (id, state, task, collision) in dubins_query.iter() {
            let terminated = task.is_dubins_terminated(state, collision);
            termination_buffer.insert(id.id.clone(), terminated);
            info!("Termination for {:?} is {:?}", id, terminated);
        }

        // Calculate termination for full aircraft
        for (id, spatial, task, collision) in full_query.iter() {
            let terminated = task.is_full_terminated(spatial, collision);
            termination_buffer.insert(id.id.clone(), terminated);
            info!("Termination for {:?} is {:?}", id, terminated);
        }
    }
}
