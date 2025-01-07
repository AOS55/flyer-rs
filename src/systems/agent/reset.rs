use crate::components::{
    AircraftState, DubinsAircraftConfig, DubinsAircraftState, FullAircraftConfig,
    FullAircraftState, PlayerController,
};
use crate::plugins::{Identifier, ResetCompleteEvent, ResetRequestEvent};
use crate::resources::AgentState;
use bevy::prelude::*;

/// System to handle resetting both Dubins and Full Aircraft state resets
pub fn reset_env(
    mut reset_events: EventReader<ResetRequestEvent>,
    mut reset_complete: EventWriter<ResetCompleteEvent>,
    mut dubins_query: Query<
        (&Identifier, &DubinsAircraftConfig, &mut DubinsAircraftState),
        With<PlayerController>,
    >,
    mut full_query: Query<
        (&Identifier, &FullAircraftConfig, &mut FullAircraftState),
        With<PlayerController>,
    >,
    mut agent_state: ResMut<AgentState>,
) {
    for _event in reset_events.read() {
        // Reset the agent state
        agent_state.reset();
        info!("In Event!");
        // Reset Dubins aircraft
        for (identifier, config, mut state) in dubins_query.iter_mut() {
            info!("Pre-reset state: {:?}", state);
            *state = DubinsAircraftState::random_position(config.random_start_config.clone());
            info!("Post-reset state: {:?}", state);

            // Update state buffer
            if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
                state_buffer.insert(identifier.id.clone(), AircraftState::Dubins(state.clone()));
            }
        }

        // Reset Full aircraft
        for (identifier, _, mut state) in full_query.iter_mut() {
            // TODO: Implement position for full aircraft
            *state = FullAircraftState::default();
            if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
                state_buffer.insert(identifier.id.clone(), AircraftState::Full(state.clone()));
            }
        }

        // Send reset complete event
        reset_complete.send(ResetCompleteEvent);
    }
}
