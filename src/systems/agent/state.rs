use bevy::prelude::*;

use crate::components::{AircraftState, DubinsAircraftState, PlayerController, SpatialComponent};
use crate::plugins::Identifier;
use crate::resources::{AgentState, AircraftAssets};

pub fn collect_state(
    dubins_query: Query<(Entity, &Identifier, &mut DubinsAircraftState), With<PlayerController>>,
    full_query: Query<(Entity, &Identifier, &mut AircraftState), With<PlayerController>>,
    agent_state: ResMut<AgentState>,
) {
    if let Ok(state_buffer) = agent_state.state_buffer.lock() {
        // Collect Dubins aircraft state

        // Collect Full aircraft state
    }
}
