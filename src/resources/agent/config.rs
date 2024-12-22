use pyo3::prelude::*;
use std::str::FromStr;

/// Modes the simulation can run in
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    RGBArray,
    Human,
}

// Utiltiy to extract render mode from Python string
impl FromStr for RenderMode {
    type Err = PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "rgb_array" => Ok(Self::RGBArray),
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Invalid render mode",
            )),
        }
    }
}

/// Configuration for the agent plugin
#[derive(Debug, Clone, Copy)]
pub struct AgentConfig {
    pub mode: RenderMode,
    pub render_width: f32,
    pub render_height: f32,
    pub frame_skip: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            mode: RenderMode::RGBArray,
            render_width: 800.0,
            render_height: 600.0,
            frame_skip: 4,
        }
    }
}
