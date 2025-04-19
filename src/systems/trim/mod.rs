pub mod handle_events;
mod problem;
pub mod solver;

pub use handle_events::handle_trim_requests;
pub use problem::{params_to_state_inputs, TrimProblem};
pub use solver::trim_aircraft_system;
