use bevy::prelude::*;

use crate::{
    components::{CollisionComponent, DubinsAircraftState, SpatialComponent, TaskComponent},
    plugins::Identifier,
    resources::AgentState,
};

pub fn calculate_reward(
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
    if let Ok(mut reward_buffer) = agent_state.reward_buffer.lock() {
        reward_buffer.clear();

        for (id, state, task, collision) in dubins_query.iter() {
            let reward_value = task.calculate_dubins_reward(state, collision) * task.weight;
            reward_buffer.insert(id.id.clone(), reward_value);
            info!("Reward for {:?} is {:?}", id, reward_value);
        }

        // Calculate rewards for full aircraft
        for (id, spatial, task, collision) in full_query.iter() {
            let reward_value = task.calculate_full_reward(spatial, collision) * task.weight;
            reward_buffer.insert(id.id.clone(), reward_value);
            info!("Reward for {:?} is {:?}", id, reward_value);
        }
    }
}
