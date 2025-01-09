use bevy::prelude::*;
use std::{collections::HashMap, io::Write};

use crate::{
    components::{AircraftControls, DubinsAircraftState, PlayerController},
    plugins::{Id, Identifier, SimState},
    resources::{AgentState, UpdateControl},
    server::{Response, ServerState, ToControls, ToObservation},
};

#[derive(Event)]
pub struct StepRequestEvent {
    pub actions: HashMap<String, HashMap<String, f64>>,
}

#[derive(Event)]
pub struct StepCompleteEvent {
    pub observations: HashMap<String, HashMap<String, f64>>,
}

#[derive(Event)]
pub struct ResetRequestEvent {
    pub seed: Option<u64>,
}

#[derive(Event)]
pub struct ResetCompleteEvent;

pub fn waiting_for_action(
    mut step_requests: EventReader<StepRequestEvent>,
    mut next_state: ResMut<NextState<SimState>>,
    mut update_control: ResMut<UpdateControl>,
    agent_state: ResMut<AgentState>,
    server: Res<ServerState>,
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
            next_state.set(SimState::RunningPhysics);
        }
    }
}

pub fn running_physics(
    mut next_state: ResMut<NextState<SimState>>,
    mut update_control: ResMut<UpdateControl>,
    agent_state: ResMut<AgentState>,
    mut dubins_query: Query<
        (Entity, &Identifier, &mut DubinsAircraftState),
        With<PlayerController>,
    >,
) {
    if update_control.remaining_steps > 0 {
        if let Ok(action_queue) = agent_state.action_queue.lock() {
            for (_entity, identifier, mut aircraft) in dubins_query.iter_mut() {
                if let Some(controls) = action_queue.get(&identifier.id) {
                    match controls {
                        AircraftControls::Dubins(dubins_controls) => {
                            info!(
                                "Applying controls to aircraft {:?}: {:?}",
                                identifier.id, dubins_controls
                            );
                            aircraft.controls = *dubins_controls;
                        }
                        _ => warn!("Received non-Dubins controls for Dubins aircraft"),
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
            next_state.set(SimState::SendingResponse);
        }
    }
}

pub fn sending_response(
    mut next_state: ResMut<NextState<SimState>>,
    agent_state: Res<AgentState>,
    server: Res<ServerState>,
) {
    if let Ok(state_buffer) = agent_state.state_buffer.lock() {
        let mut all_observations = HashMap::new();

        // Collect observations from all aircraft
        for (id, state) in state_buffer.iter() {
            let id_str = match id {
                Id::Named(name) => name.clone(),
                Id::Entity(entity) => entity.to_string(),
            };

            if let Some(obs_space) = server.config.observation_spaces.get(&id_str) {
                let obs = obs_space.to_observation(state);
                all_observations.insert(id_str, obs);
            }
        }

        // Send response via TCP
        if !all_observations.is_empty() {
            if let Ok(guard) = server.conn.lock() {
                if let Ok(mut stream) = guard.try_clone() {
                    let response = Response {
                        obs: all_observations,
                        reward: 0.0,
                        terminated: false,
                        truncated: false,
                        info: serde_json::json!({}),
                    };

                    let response_str = serde_json::to_string(&response).unwrap_or_default() + "\n";

                    if stream.write_all(response_str.as_bytes()).is_ok() {
                        stream.flush().ok();
                        info!("Response sent successfully");
                    } else {
                        error!("Failed to send response");
                    }
                }
            }
        } else {
            warn!("No observations to send");
        }
    }

    // Return to waiting state
    next_state.set(SimState::WaitingForAction);
}

pub fn handle_reset_response(
    mut reset_complete: EventReader<ResetCompleteEvent>,
    agent_state: Res<AgentState>,
    server: Res<ServerState>,
    mut next_state: ResMut<NextState<SimState>>,
) {
    for _ in reset_complete.read() {
        if let Ok(state_buffer) = agent_state.state_buffer.lock() {
            let mut all_observations = HashMap::new();

            // Collect initial observations
            for (id, state) in state_buffer.iter() {
                let id_str = match id {
                    Id::Named(name) => name.clone(),
                    Id::Entity(entity) => entity.to_string(),
                };

                if let Some(obs_space) = server.config.observation_spaces.get(&id_str) {
                    let obs = obs_space.to_observation(state);
                    all_observations.insert(id_str, obs);
                }
            }

            // Send reset response
            if let Ok(guard) = server.conn.lock() {
                if let Ok(mut stream) = guard.try_clone() {
                    let response = Response {
                        obs: all_observations,
                        reward: 0.0,
                        terminated: false,
                        truncated: false,
                        info: serde_json::json!({}),
                    };

                    if let Ok(response_str) = serde_json::to_string(&response) {
                        if stream.write_all((response_str + "\n").as_bytes()).is_ok() {
                            stream.flush().ok();
                            // Transition back to waiting state
                            next_state.set(SimState::WaitingForAction);
                        }
                    }
                }
            }
        }
    }
}
