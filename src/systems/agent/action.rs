use bevy::prelude::*;

use crate::components::{AircraftControls, AircraftState, DubinsAircraftState, PlayerController};
use crate::plugins::Identifier;
use crate::resources::AgentState;

// System for applying agent actions
pub fn apply_action(
    mut dubins_query: Query<
        (Entity, &Identifier, &mut DubinsAircraftState),
        With<PlayerController>,
    >,
    mut full_query: Query<(Entity, &Identifier, &mut AircraftState), With<PlayerController>>,
    agent_state: Res<AgentState>,
) {
    if let Ok(action_queue) = agent_state.action_queue.lock() {
        // Handle Dubins aircraft
        for (entity, identifier, mut aircraft) in dubins_query.iter_mut() {
            if let Some(AircraftControls::Dubins { .. }) = action_queue.get(&identifier.id) {
                // Apply Dubins controls
            }
        }

        // Handle Full aircraft
        for (entity, identifier, mut aircraft) in full_query.iter_mut() {
            if let Some(AircraftControls::Full { .. }) = action_queue.get(&identifier.id) {
                // Apply Full controls
            }
        }
    }
}
