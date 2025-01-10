use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::{plugins::SimState, server::config::EnvConfig};

/// Enum representing commands sent to the server.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Command {
    /// Initialize the environment with a configuration.
    Initialize { config: serde_json::Value },
    /// Perform a simulation step with provided actions.
    Step {
        actions: HashMap<String, HashMap<String, f64>>,
    },
    /// Reset the environment with an optional random seed.
    Reset { seed: Option<u64> },
    /// Close the server connection.
    Close,
}

/// Resource representing the server state.
#[derive(Resource)]
pub struct ServerState {
    /// Connection to the client.
    pub conn: Arc<Mutex<TcpStream>>,
    /// Whether the server is initialized.
    pub initialized: bool,
    /// Configuration of the environment.
    pub config: EnvConfig,
    /// simulation state
    pub sim_state: SimState,
}

/// Struct representing the response from the server after handling a command.
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    /// Observation data from the environment (for each aircraft).
    pub obs: HashMap<String, HashMap<String, f64>>,
    /// Reward for the current step.
    pub reward: f64,
    /// Whether the episode is terminated.
    pub terminated: bool,
    /// Whether the episode is truncated.
    pub truncated: bool,
    /// Additional info about the step or environment state.
    pub info: serde_json::Value,
}
