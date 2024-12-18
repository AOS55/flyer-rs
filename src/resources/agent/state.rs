use crate::components::AircraftControls;
use crate::plugins::Id;
use crate::resources::agent::config::{AgentConfig, SimulationMode};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Resource)]
pub struct AgentState {
    // Time Tracking
    pub episode_count: u32,
    pub current_step: u32,

    pub state_buffer: Arc<Mutex<HashMap<Id, Vec<f64>>>>,
    pub action_queue: Arc<Mutex<HashMap<Id, AircraftControls>>>,
    pub render_buffer: Arc<Mutex<Option<Vec<u8>>>>,

    pub mode: SimulationMode,
    pub terminated: bool,
    pub truncated: bool,
}

impl AgentState {
    pub fn new(config: &AgentConfig) -> Self {
        Self {
            episode_count: 0,
            current_step: 0,
            state_buffer: Arc::new(Mutex::new(HashMap::new())),
            action_queue: Arc::new(Mutex::new(HashMap::new())),
            render_buffer: Arc::new(Mutex::new(None)),
            mode: config.mode,
            terminated: false,
            truncated: false,
        }
    }
}
