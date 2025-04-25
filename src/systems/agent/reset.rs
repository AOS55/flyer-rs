use bevy::prelude::*;
use std::{collections::HashMap, io::Write};

use crate::{
    components::{
        AirData, AircraftConfig, AircraftControlSurfaces, AircraftState, DubinsAircraftConfig,
        DubinsAircraftState, FullAircraftConfig, FullAircraftState, PhysicsComponent,
        PlayerController, PropulsionState, SpatialComponent,
    },
    plugins::{Id, Identifier, ResetCompleteEvent, ResetRequestEvent, SimState},
    resources::AgentState,
    server::{Response, ServerState, ToObservation},
};

pub fn handle_reset_response(
    mut reset_complete: EventReader<ResetCompleteEvent>,
    // agent_state: Res<AgentState>, // REMOVE - No longer read buffer here
    mut server: ResMut<ServerState>,
) {
    let conn = server.conn.clone();
    // info!("Handling Reset Response");

    // Process the event payload directly
    for event in reset_complete.read() {
        // info!("Processing ResetCompleteEvent received from reset_env");

        // Use observations directly from the event
        let all_observations = event.initial_observations.clone();

        // Initial reward and termination are typically zero/false after reset
        // Create empty maps for these for the response structure
        let all_rewards: HashMap<String, f64> = HashMap::new();
        let all_terminations: HashMap<String, bool> = HashMap::new();

        // Log detailed observation info for debugging
        // info!("Observations received via event: {:?}", all_observations);

        // Send reset response
        if let Ok(guard) = conn.lock() {
            if let Ok(mut stream) = guard.try_clone() {
                let response = Response {
                    obs: all_observations,        // Use observations from event
                    reward: all_rewards,          // Empty map for initial reset
                    terminated: all_terminations, // Empty map for initial reset
                    truncated: false,             // Reset never truncates initially
                    info: serde_json::json!({}),  // Empty info dict for now
                };

                match serde_json::to_string(&response) {
                    Ok(response_str) => {
                        // info!("Sending reset response: {}", response_str);
                        if stream.write_all((response_str + "\n").as_bytes()).is_ok() {
                            if let Err(e) = stream.flush() {
                                error!(
                                    "Failed to flush stream after sending reset response: {}",
                                    e
                                );
                            }
                            // Transition back to waiting state ONLY after successful send+flush
                            // info!("Reset successful, transitioning to WaitingForAction");
                            server.sim_state = SimState::WaitingForAction;
                        } else {
                            error!("Failed to write reset response to stream");
                            // Consider what state to be in if writing fails - maybe retry? Stay Resetting?
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize reset response: {}", e);
                        // Consider state transition on serialization failure
                    }
                }
            } else {
                error!("Failed to clone stream for reset response");
                // Consider state transition on clone failure
            }
        } else {
            error!("Failed to lock connection for reset response");
            // Consider state transition on lock failure
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
            &mut DubinsAircraftConfig, // Keep config for potential updates
            &mut DubinsAircraftState,
        ),
        With<PlayerController>,
    >,
    mut full_query: Query<
        (
            &Identifier,
            &mut FullAircraftConfig, // Keep config for potential updates
            &mut AirData,
            &mut AircraftControlSurfaces,
            &mut SpatialComponent,
            &mut PhysicsComponent,
            &mut PropulsionState,
        ),
        With<PlayerController>,
    >,
    mut agent_state: ResMut<AgentState>,
    mut server: ResMut<ServerState>, // Needs to be mutable to update config
) {
    // info!("Resetting agent state");

    for event in reset_events.read() {
        // Reset the agent state buffers (except action queue)
        agent_state.reset(); // Make sure all buffers are cleared

        // 1. Rebuild config if seed is provided
        if let Some(seed) = event.seed {
            match server.config.rebuild_with_seed(seed) {
                Ok(new_config) => {
                    server.config = new_config;
                    // info!("Successfully rebuilt EnvConfig with seed: {}", seed);
                    // Log success
                }
                Err(e) => {
                    error!("Failed to rebuild config with seed {}: {}", seed, e);
                    continue; // Skip this reset event if rebuild fails
                }
            }
        }

        // --- Start Modification ---
        let mut initial_observations = HashMap::new(); // Map to store initial obs
        let observation_spaces = &server.config.observation_spaces; // Borrow reference to avoid cloning inside loop

        // 2. Populate state_buffer directly from the potentially updated config AND generate initial obs
        if let Ok(mut state_buffer) = agent_state.state_buffer.lock() {
            state_buffer.clear(); // Ensure it's empty before populating

            for (id_str, config) in &server.config.aircraft_configs {
                let initial_state = match config {
                    AircraftConfig::Dubins(dubins_conf) => {
                        // info!("Generating initial Dubins state for {} from config", id_str);
                        AircraftState::Dubins(DubinsAircraftState::from_config(
                            &dubins_conf.start_config,
                        ))
                    }
                    AircraftConfig::Full(full_conf) => {
                        // info!("Generating initial Full state for {} from config", id_str);
                        AircraftState::Full(FullAircraftState::from_config(full_conf))
                    }
                };
                let id = Id::Named(id_str.clone()); // Assuming IDs are strings for now

                // Populate buffer for subsequent steps' state collection
                state_buffer.insert(id, initial_state.clone());
                // info!("Populated state buffer for {}", id_str);

                // ---> Generate the initial observation for the reset response <---
                if let Some(obs_space) = observation_spaces.get(id_str) {
                    let obs = obs_space.to_observation(&initial_state);
                    // info!("Generated initial observation for {}: {:?}", id_str, obs);
                    initial_observations.insert(id_str.clone(), obs);
                } else {
                    error!(
                        "No observation space found for {} during reset generation!",
                        id_str
                    );
                    // Insert empty map to prevent potential downstream issues if key is expected
                    initial_observations.insert(id_str.clone(), HashMap::new());
                }
                // ---> End observation generation <---
            }

            // info!(
            //     "State buffer populated with {} aircraft",
            //     state_buffer.len()
            // );
            // let keys: Vec<_> = state_buffer.keys().cloned().collect(); // Clone keys for logging
            // info!("Populated state buffer contains keys: {:?}", keys);
        } else {
            error!("Failed to lock state buffer for reset population.");
            continue; // Skip if lock fails
        }

        // Check if observations were generated (important if configs exist)
        if initial_observations.is_empty() && !server.config.aircraft_configs.is_empty() {
            error!("Failed to generate any initial observations despite having aircraft configs!");
            // Depending on requirements, you might want to `continue;` here
        }
        // --- End Modification ---

        // 3. Update actual entities in the ECS world (if they exist yet)
        // (Keep existing logic for updating Dubins/Full entities)
        // Reset Dubins aircraft entities
        for (identifier, mut dubins_config_comp, mut state_comp) in dubins_query.iter_mut() {
            if let Some(config) = server.config.aircraft_configs.get(&identifier.to_string()) {
                match config {
                    AircraftConfig::Dubins(ref new_dubins_config) => {
                        // info!("Resetting Dubins entity: {}", identifier.to_string());
                        *dubins_config_comp = new_dubins_config.clone();
                        *state_comp =
                            DubinsAircraftState::from_config(&new_dubins_config.start_config);
                    }
                    _ => error!(
                        "Mismatched config type for Dubins entity {}",
                        identifier.to_string()
                    ),
                }
            } else {
                warn!(
                    "Config not found for Dubins entity during reset: {}",
                    identifier.to_string()
                );
            }
        }

        // Reset Full aircraft entities
        for (
            identifier,
            mut full_config_comp,
            mut air_data,
            mut control_surfaces,
            mut spatial_comp,
            mut physics_comp,
            mut propulsion_comp,
        ) in full_query.iter_mut()
        {
            if let Some(config) = server.config.aircraft_configs.get(&identifier.to_string()) {
                match config {
                    AircraftConfig::Full(ref new_full_config) => {
                        // info!("Resetting Full entity: {}", identifier.to_string());
                        *full_config_comp = new_full_config.clone();
                        let new_state = FullAircraftState::from_config(new_full_config);
                        *spatial_comp = new_state.spatial;
                        *physics_comp = new_state.physics;
                        *air_data = new_state.air_data;
                        *control_surfaces = new_state.control_surfaces;
                        // Reset propulsion state based on the actual number of engines
                        *propulsion_comp =
                            PropulsionState::new(new_full_config.propulsion.engines.len());

                        // info!(
                        // "Reset Full entity {} complete. Components updated.",
                        // identifier.to_string()
                        // );
                    }
                    _ => error!(
                        "Mismatched config type for Full entity {}",
                        identifier.to_string()
                    ),
                }
            } else {
                warn!(
                    "Config not found for Full entity during reset: {}",
                    identifier.to_string()
                );
            }
        }

        // 4. Send reset complete event WITH initial observations
        // info!("Sending ResetCompleteEvent with initial observations");
        reset_complete.send(ResetCompleteEvent {
            initial_observations,
        }); // <-- Pass the generated map
    }
}
