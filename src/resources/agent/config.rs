/// Modes the simulation can run in
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimulationMode {
    RGBArray,
    Human,
}

/// Configuration for the agent plugin
#[derive(Debug, Clone, Copy)]
pub struct AgentConfig {
    pub mode: SimulationMode,
    pub render_width: u32,
    pub render_height: u32,
    pub frame_skip: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            mode: SimulationMode::RGBArray,
            render_width: 800,
            render_height: 600,
            frame_skip: 4,
        }
    }
}
