use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
    #[error("State validation failed: {0}")]
    ValidationFailed(String),
    #[error("State update error: {0}")]
    UpdateError(String),
}
