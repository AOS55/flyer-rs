use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents the current state of an engine
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PowerplantState {
    /// Current power lever setting (0.0 to 1.0)
    pub power_lever: f64,
    /// Current thrust as a fraction of maximum thrust (0.0 to 1.0)
    pub thrust_fraction: f64,
    /// Current fuel flow rate (kg/s)
    pub fuel_flow: f64,
    /// Engine running status
    pub running: bool,
}

/// Component to track the state of all engines
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PropulsionState {
    /// State of each engine
    pub engine_states: Vec<PowerplantState>,
}

impl Default for PowerplantState {
    fn default() -> Self {
        Self {
            power_lever: 0.0,
            thrust_fraction: 0.0,
            fuel_flow: 0.0,
            running: false,
        }
    }
}
impl Default for PropulsionState {
    fn default() -> Self {
        Self {
            engine_states: vec![PowerplantState::default()],
        }
    }
}

impl PropulsionState {
    /// Creates a new PropulsionState with the specified number of engines
    pub fn new(num_engines: usize) -> Self {
        Self {
            engine_states: vec![PowerplantState::default(); num_engines],
        }
    }

    /// Sets the power_levers for all engines
    pub fn set_throttle(&mut self, power_lever: f64) {
        let power_lever = power_lever.clamp(0.0, 1.0);
        for state in &mut self.engine_states {
            state.power_lever = power_lever;
        }
    }

    /// Sets the throttle for a specific engine
    pub fn set_engine_throttle(&mut self, engine_index: usize, throttle: f64) {
        if let Some(state) = self.engine_states.get_mut(engine_index) {
            state.power_lever = throttle.clamp(0.0, 1.0);
        }
    }
}
