use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("Asset error: {0}")]
    Asset(#[from] super::assets::AssetError),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Resource type mismatch: {0}")]
    TypeMismatch(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
}

impl From<Box<dyn Error>> for ResourceError {
    fn from(err: Box<dyn Error>) -> Self {
        ResourceError::Config(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ResourceError>;
