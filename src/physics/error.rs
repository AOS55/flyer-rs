use crate::state::StateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PhysicsError {
    #[error("State error: {0}")]
    StateError(#[from] StateError),

    #[error("Physics computation error: {0}")]
    ComputationError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Model configuration error: {0}")]
    ConfigError(String),
}
