use crate::rendering::RenderError;
use crate::state::StateError;
use crate::state::StateManager;

pub mod aircraft;
pub mod state;

pub trait Vehicle: StateManager {
    fn update(&mut self, dt: f64) -> Result<(), StateError>;
    fn render(&self) -> Result<(), RenderError>;
}
