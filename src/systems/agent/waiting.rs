use bevy::prelude::*;

use crate::{
    plugins::{Id, SimState, StepRequestEvent},
    resources::{AgentState, UpdateControl},
    server::{ServerState, ToControls},
};

pub fn waiting_for_action(
    mut step_requests: EventReader<StepRequestEvent>,
    mut update_control: ResMut<UpdateControl>,
    agent_state: ResMut<AgentState>,
    mut server: ResMut<ServerState>,
) {
    if update_control.remaining_steps == 0 {
        for request in step_requests.read() {
            info!("Processing new requested action request");

            if let Ok(mut action_queue) = agent_state.action_queue.lock() {
                for (aircraft_id, action) in &request.actions {
                    if let Some(action_space) = server.config.action_spaces.get(aircraft_id) {
                        let controls = action_space.to_controls(action.clone());
                        let id = Id::Named(aircraft_id.to_string());
                        action_queue.insert(id, controls);
                        info!("Queued action for aircraft {}: {:?}", aircraft_id, controls);
                    }
                }
            }

            // Set number of physics steps to run
            update_control.set_steps(server.config.steps_per_action);
            info!(
                "Set {}, physics steps to run",
                server.config.steps_per_action
            );

            // Transition to physics state
            server.sim_state = SimState::RunningPhysics;
            info!("state: {:?}", server.sim_state);
        }
    }
}
