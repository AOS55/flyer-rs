use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
    #[error("Invalid value error: {0}")]
    InvalidValue(String),
    #[error("State validation failed: {0}")]
    ValidationFailed(String),
    #[error("State synchronization error: {0}")]
    SyncError(String),
}
