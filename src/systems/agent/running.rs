use bevy::prelude::*;

use crate::{
    components::{
        AircraftControlSurfaces, AircraftControls, DubinsAircraftState, PlayerController,
    },
    plugins::{Identifier, SimState},
    resources::{AgentState, UpdateControl},
    server::ServerState,
};

pub fn running_physics(
    mut update_control: ResMut<UpdateControl>,
    agent_state: ResMut<AgentState>,
    mut server: ResMut<ServerState>,
    mut dubins_query: Query<
        (Entity, &Identifier, &mut DubinsAircraftState),
        With<PlayerController>,
    >,
    mut full_query: Query<
        (Entity, &Identifier, &mut AircraftControlSurfaces),
        With<PlayerController>,
    >,
) {
    info!("Running physics");
    if update_control.remaining_steps > 0 {
        if let Ok(action_queue) = agent_state.action_queue.lock() {
            for (_entity, identifier, mut aircraft) in dubins_query.iter_mut() {
                if let Some(controls) = action_queue.get(&identifier.id) {
                    match controls {
                        AircraftControls::Dubins(dubins_controls) => {
                            info!(
                                "Applying controls to dubins aircraft {:?}: {:?}",
                                identifier.id, dubins_controls
                            );
                            aircraft.controls = *dubins_controls;
                        }
                        _ => warn!("Received non-Dubins controls for Dubins aircraft"),
                    }
                }
            }

            for (_entity, identifier, mut control_surfaces) in full_query.iter_mut() {
                if let Some(controls) = action_queue.get(&identifier.id) {
                    match controls {
                        AircraftControls::Full(full_controls) => {
                            info!(
                                "Applying controls to full aircraft {:?}: {:?}",
                                identifier.id, full_controls
                            );
                            control_surfaces.aileron = full_controls.aileron;
                            control_surfaces.elevator = full_controls.elevator;
                            control_surfaces.rudder = full_controls.rudder;
                            control_surfaces.power_lever = full_controls.power_lever;
                        }
                        _ => warn!("Received non-Full controls for Full aircraft"),
                    }
                }
            }
        }

        // Consume one physics step
        update_control.consume_step();
        info!(
            "Physics step complete, {} steps remaining",
            update_control.remaining_steps
        );

        // If this was the last step, transition to response state
        if update_control.remaining_steps == 0 {
            info!("Physics steps complete, transitioning to response state");
            server.sim_state = SimState::SendingResponse;
        }
    }
}
