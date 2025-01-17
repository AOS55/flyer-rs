use bevy::prelude::*;
use std::{collections::HashMap, io::Write};

use crate::{
    plugins::{Id, SimState},
    resources::AgentState,
    server::{Response, ServerState, ToObservation},
};

pub fn sending_response(agent_state: Res<AgentState>, mut server: ResMut<ServerState>) {
    if let (Ok(state_buffer), Ok(reward_buffer)) = (
        agent_state.state_buffer.lock(),
        agent_state.reward_buffer.lock(),
    ) {
        let mut all_observations = HashMap::new();
        let mut all_rewards = HashMap::new();

        // Collect observations from all aircraft
        for (id, state) in state_buffer.iter() {
            let id_str = match id {
                Id::Named(name) => name.clone(),
                Id::Entity(entity) => entity.to_string(),
            };

            // Get observations
            if let Some(obs_space) = server.config.observation_spaces.get(&id_str) {
                let obs = obs_space.to_observation(state);
                all_observations.insert(id_str.clone(), obs);
            }

            // Get rewards
            if let Some(&reward) = reward_buffer.get(id) {
                all_rewards.insert(id_str.clone(), reward);
            }
        }

        // Send response via TCP
        if !all_observations.is_empty() {
            if let Ok(guard) = server.conn.lock() {
                if let Ok(mut stream) = guard.try_clone() {
                    let response = Response {
                        obs: all_observations,
                        reward: all_rewards,
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
    server.sim_state = SimState::WaitingForAction;
}
