use bevy::prelude::*;
use std::{collections::HashMap, io::Write};

use crate::{
    components::{
        AirData, AircraftConfig, AircraftControlSurfaces, AircraftState, DubinsAircraftConfig,
        DubinsAircraftState, FullAircraftConfig, FullAircraftState, PhysicsComponent,
        PlayerController, PowerplantState, SpatialComponent, StartConfig,
    },
    plugins::{Id, Identifier, ResetCompleteEvent, ResetRequestEvent, SimState},
    resources::AgentState,
    server::{Response, ServerState, ToObservation},
};

pub fn handle_reset_response(
    mut reset_complete: EventReader<ResetCompleteEvent>,
    agent_state: Res<AgentState>,
    mut server: ResMut<ServerState>,
) {
    let observation_spaces = &server.config.observation_spaces.clone();
    let conn = server.conn.clone();
    info!("Handling Reset Response");
    for _ in reset_complete.read() {
        if let (Ok(state_buffer), Ok(termination_buffer), Ok(reward_buffer)) = (
            agent_state.state_buffer.lock(),
            agent_state.termination_buffer.lock(),
            agent_state.reward_buffer.lock(),
        ) {
            let mut all_observations = HashMap::new();
            let mut all_terminations = HashMap::new();
            let mut all_rewards = HashMap::new();

            // Collect initial observations
            for (id, state) in state_buffer.iter() {
                info!("reset_response, id: {:?}, state: {:?}", id, state);
                let id_str = match id {
                    Id::Named(name) => name.clone(),
                    Id::Entity(entity) => entity.to_string(),
                };

                if let Some(obs_space) = observation_spaces.get(&id_str) {
                    let obs = obs_space.to_observation(state);
                    all_observations.insert(id_str.clone(), obs);
                }

                if let Some(&terminated) = termination_buffer.get(id) {
                    all_terminations.insert(id_str.clone(), terminated);
                }

                // Should be reset to 0 after reset
                if let Some(&reward) = reward_buffer.get(id) {
                    all_rewards.insert(id_str.clone(), reward);
                }
            }

            // Send reset response
            if let Ok(guard) = conn.lock() {
                if let Ok(mut stream) = guard.try_clone() {
                    let response = Response {
                        obs: all_observations,
                        reward: all_rewards,
                        terminated: all_terminations,
                        truncated: false,
                        info: serde_json::json!({}),
                    };

                    if let Ok(response_str) = serde_json::to_string(&response) {
                        if stream.write_all((response_str + "\n").as_bytes()).is_ok() {
                            stream.flush().ok();
                            // Transition back to waiting state
                            server.sim_state = SimState::WaitingForAction;
                        }
                    }
                }
            }
        }
    }
}

/// System to handle resetting environment
pub fn reset_env(
    mut reset_events: EventReader<ResetRequestEvent>,
    mut reset_complete: EventWriter<ResetCompleteEvent>,
    mut dubins_query: Query<
        (
            &Identifier,
            &mut DubinsAircraftConfig,
            &mut DubinsAircraftState,
        ),
        With<PlayerController>,
    >,
    mut full_query: Query<
        (
            &Identifier,
            &mut FullAircraftConfig,
            &AirData,
            &AircraftControlSurfaces,
            &SpatialComponent,
            &PhysicsComponent,
            &PowerplantState,
        ),
        With<PlayerController>,
    >,
    mut agent_state: ResMut<AgentState>,
    mut server: ResMut<ServerState>,
) {
    for event in reset_events.read() {
        // Reset the agent state
        agent_state.reset();

        // If seed provided, rebuild entire configuration
        if let Some(seed) = event.seed {
            match server.config.rebuild_with_seed(seed) {
                Ok(new_config) => {
                    server.config = new_config;
                }
                Err(e) => {
                    error!("Failed to rebuild config with seed {}: {}", seed, e);
                    continue;
                }
            }
        }

        // Reset Dubins aircraft using the potentially updated config
        for (identifier, _, mut state) in dubins_query.iter_mut() {
            if let Some(config) = server.config.aircraft_configs.get(&identifier.to_string()) {
                match config {
                    AircraftConfig::Dubins(config) => {
                        *state = DubinsAircraftState::from_config(&config.start_config);

                        // Update state buffer
                        if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
                            state_buffer.insert(
                                identifier.id.clone(),
                                AircraftState::Dubins(state.clone()),
                            );
                        }
                    }
                    _ => error!("Mismatched aircraft type for {}", identifier.to_string()),
                }
            }
        }

        // Reset Full aircraft using the potentially updated config
        for (identifier, _, _, _, _, _, _) in full_query.iter_mut() {
            if let Some(config) = server.config.aircraft_configs.get(&identifier.to_string()) {
                match config {
                    AircraftConfig::Full(config) => {
                        let state = FullAircraftState::from_config(&config);
                        if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
                            state_buffer.insert(identifier.id.clone(), AircraftState::Full(state));
                        }
                    }
                    _ => error!("Mismatched aircraft type for {}", identifier.to_string()),
                }
            }
        }

        // Send reset complete event
        reset_complete.send(ResetCompleteEvent);
    }
}

// pub fn reset_env(
//     mut reset_events: EventReader<ResetRequestEvent>,
//     mut reset_complete: EventWriter<ResetCompleteEvent>,
//     mut dubins_query: Query<
//         (
//             &Identifier,
//             &mut DubinsAircraftConfig,
//             &mut DubinsAircraftState,
//         ),
//         With<PlayerController>,
//     >,
//     mut full_query: Query<
//         (
//             &Identifier,
//             &mut FullAircraftConfig,
//             &AirData,
//             &AircraftControlSurfaces,
//             &SpatialComponent,
//             &PhysicsComponent,
//             &PowerplantState,
//         ),
//         With<PlayerController>,
//     >,
//     mut agent_state: ResMut<AgentState>,
// ) {
//     for event in reset_events.read() {
//         // Reset the agent state
//         agent_state.reset();

//         // Reset Dubins aircraft
//         for (identifier, mut config, mut state) in dubins_query.iter_mut() {
//             if let Some(seed) = event.seed {
//                 // Update the start config seed
//                 match &mut config.start_config {
//                     StartConfig::Random(random_config) => {
//                         random_config.seed = Some(seed);
//                     }
//                     _ => {}
//                 }
//             }

//             *state = DubinsAircraftState::from_config(&config.start_config);

//             // Update state buffer
//             if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
//                 state_buffer.insert(identifier.id.clone(), AircraftState::Dubins(state.clone()));
//             }
//         }

//         // Reset Full aircraft
//         for (identifier, mut config, _, _, _, _, _) in full_query.iter_mut() {
//             if let Some(seed) = event.seed {
//                 match &mut config.start_config {
//                     StartConfig::Random(random_config) => {
//                         random_config.seed = Some(seed);
//                     }
//                     _ => {}
//                 }
//             }
//             let state = FullAircraftState::from_config(&config);
//             if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
//                 state_buffer.insert(identifier.id.clone(), AircraftState::Full(state));
//             }
//         }

//         // Send reset complete event
//         reset_complete.send(ResetCompleteEvent);
//     }
// }
