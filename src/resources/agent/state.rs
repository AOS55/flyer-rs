use crate::components::{AircraftControls, AircraftState};
use crate::plugins::Id;
use crate::resources::agent::config::{AgentConfig, RenderMode};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Resource)]
pub struct AgentState {
    // Time Tracking
    pub sim_time: f64,
    pub episode_count: u32,
    pub current_step: u32,

    pub state_buffer: Arc<Mutex<HashMap<Id, AircraftState>>>,
    pub reward_buffer: Arc<Mutex<HashMap<Id, f64>>>,
    pub action_queue: Arc<Mutex<HashMap<Id, AircraftControls>>>,
    pub render_buffer: Arc<Mutex<Option<Vec<u8>>>>,

    pub mode: RenderMode,
    pub terminated: bool,
    pub truncated: bool,
}

impl AgentState {
    pub fn new(config: &AgentConfig) -> Self {
        Self {
            sim_time: 0.0,
            episode_count: 0,
            current_step: 0,
            state_buffer: Arc::new(Mutex::new(HashMap::new())),
            reward_buffer: Arc::new(Mutex::new(HashMap::new())),
            action_queue: Arc::new(Mutex::new(HashMap::new())),
            render_buffer: Arc::new(Mutex::new(None)),
            mode: config.mode,
            terminated: false,
            truncated: false,
        }
    }

    pub fn reset(&mut self) {
        self.episode_count += 1;
        self.current_step = 0;

        // Get existing Ids before clearing
        let aircraft_ids = if let Ok(state_buffer) = self.state_buffer.lock() {
            state_buffer.keys().cloned().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if let Ok(mut buffer) = self.state_buffer.lock() {
            buffer.clear();
        }

        if let Ok(mut rewards) = self.reward_buffer.lock() {
            rewards.clear();
            // Reinitialize rewards to 0.0 for each aircraft
            for id in aircraft_ids {
                rewards.insert(id, 0.0);
            }
        }

        if let Ok(mut queue) = self.action_queue.lock() {
            queue.clear();
        }

        self.terminated = false;
        self.truncated = false;
    }

    pub fn update_time(&mut self, dt: f64) {
        self.sim_time += dt;
    }
}
