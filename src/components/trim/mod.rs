mod config;
mod event;
mod solver;
mod state;

pub use config::{TrimBounds, TrimSolverConfig};
pub use event::{NeedsTrim, TrimRequest};
pub use solver::TrimSolver;
pub use state::{TrimCondition, TrimResiduals, TrimResult, TrimState, TrimStateConversion};
