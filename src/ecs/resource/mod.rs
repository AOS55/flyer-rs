mod manager;

pub use manager::ResourceManager;
// use std::any::Any;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
    #[error("Resource type mismatch: {0}")]
    TypeMismatch(String),
}

pub trait Resource: Send + Sync {}

impl<T: Send + Sync + 'static> Resource for T {}
