use bevy::prelude::*;

use crate::components::{AircraftState, DubinsAircraftState, FullAircraftState, PlayerController};
use crate::plugins::Identifier;
use crate::resources::AgentState;

pub fn collect_state(
    dubins_query: Query<(Entity, &Identifier, &mut DubinsAircraftState), With<PlayerController>>,
    full_query: Query<(Entity, &Identifier, &mut FullAircraftState), With<PlayerController>>,
    agent_state: ResMut<AgentState>,
) {
    if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
        state_buffer.clear();

        // Collect Dubins aircraft state
        for (_entity, identifier, dubins_state) in dubins_query.iter() {
            state_buffer.insert(
                identifier.id.clone(),
                AircraftState::Dubins(dubins_state.clone()),
            );
        }

        // Collect Full aircraft state
        for (_entity, identifier, full_state) in full_query.iter() {
            state_buffer.insert(
                identifier.id.clone(),
                AircraftState::Full(full_state.clone()),
            );
        }
    }
}
