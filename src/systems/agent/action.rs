use bevy::prelude::*;

use crate::components::{
    AircraftControls, DubinsAircraftState, FullAircraftState, PlayerController,
};
use crate::plugins::Identifier;
use crate::resources::AgentState;

/// System for applying agent actions to aircraft.
///
/// This system processes queued actions from agents (e.g., AI or players) and applies
/// them to the appropriate aircraft. It distinguishes between Dubins and Full aircraft
/// models and ensures the correct controls are applied to each type.
pub fn apply_action(
    mut dubins_query: Query<
        (Entity, &Identifier, &mut DubinsAircraftState),
        With<PlayerController>,
    >,
    mut full_query: Query<(Entity, &Identifier, &mut FullAircraftState), With<PlayerController>>,
    agent_state: Res<AgentState>,
) {
    // Access the action queue shared among agents
    if let Ok(action_queue) = agent_state.action_queue.lock() {
        // Handle Dubins aircraft
        for (_entity, identifier, mut aircraft) in dubins_query.iter_mut() {
            if let Some(controls) = action_queue.get(&identifier.id) {
                match controls {
                    AircraftControls::Dubins(dubins_controls) => {
                        // Apply Dubins controls directly to the aircraft state
                        info!("Apply Action: {:?}", dubins_controls);
                        aircraft.controls = *dubins_controls;
                        info!("Aircraft Controls: {:?}", aircraft.controls);
                    }
                    AircraftControls::Full(_) => {
                        warn!(
                            "Received Full aircraft controls for Dubins aircraft: {:?}",
                            identifier.id
                        );
                    }
                }
            }
        }

        // Handle Full aircraft
        for (_entity, identifier, mut aircraft) in full_query.iter_mut() {
            if let Some(controls) = action_queue.get(&identifier.id) {
                match controls {
                    AircraftControls::Full(full_controls) => {
                        // Apply Full controls to the aircraft control surfaces
                        aircraft.control_surfaces = *full_controls;
                    }
                    AircraftControls::Dubins(_) => {
                        warn!(
                            "Received Dubins controls for Full aircraft: {:?}",
                            identifier.id
                        );
                    }
                }
            }
        }
    }
}
