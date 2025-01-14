use bevy::prelude::*;

use crate::components::{
    AirData, AircraftControlSurfaces, AircraftState, DubinsAircraftState, FullAircraftState,
    PhysicsComponent, PlayerController, PowerplantState, SpatialComponent,
};
use crate::plugins::Identifier;
use crate::resources::AgentState;

/// System for collecting the current state of aircraft and storing it in a shared buffer.
///
/// This system gathers the state of both Dubins and Full aircraft models associated with
/// player-controlled entities and updates the `AgentState` resource. The collected data
/// can be used by agents (e.g., AI or player control logic) for decision-making.
pub fn collect_state(
    dubins_query: Query<(Entity, &Identifier, &mut DubinsAircraftState), With<PlayerController>>,
    full_query: Query<
        (
            Entity,
            &Identifier,
            &AirData,
            &AircraftControlSurfaces,
            &SpatialComponent,
            &PhysicsComponent,
            &PowerplantState,
        ),
        With<PlayerController>,
    >,
    agent_state: ResMut<AgentState>,
) {
    // Access the shared state buffer in the `AgentState` resource
    if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
        // Clear the buffer to prepare for fresh state data
        state_buffer.clear();

        // Collect state from Dubins aircraft
        for (_entity, identifier, dubins_state) in dubins_query.iter() {
            state_buffer.insert(
                identifier.id.clone(),
                AircraftState::Dubins(dubins_state.clone()),
            );
        }

        // Collect state from Full aircraft
        for (_entity, identifier, air_data, control_surfaces, spatial, physics, powerplant) in
            full_query.iter()
        {
            let full_state = FullAircraftState {
                air_data: air_data.clone(),
                control_surfaces: control_surfaces.clone(),
                spatial: spatial.clone(),
                physics: physics.clone(),
                propulsion: powerplant.clone(),
            };

            state_buffer.insert(identifier.id.clone(), AircraftState::Full(full_state));
        }
    }
}
