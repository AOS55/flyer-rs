use serde::{Deserialize, Serialize};

/// Modes the simulation can run in
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RenderMode {
    RGBArray,
    Human,
}

/// Configuration for the agent plugin
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AgentConfig {
    pub mode: RenderMode,
    pub render_width: f32,
    pub render_height: f32,
    pub frame_skip: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            mode: RenderMode::Human,
            render_width: 800.0,
            render_height: 600.0,
            frame_skip: 4,
        }
    }
}
