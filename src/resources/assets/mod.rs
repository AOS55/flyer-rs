mod loader;
mod manager;

pub use manager::{AssetManager, AssetType};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Asset not found: {0}")]
    NotFound(String),
    #[error("Type mismatch for asset: {0}")]
    TypeMismatch(String),
    #[error("Failed to load asset: {0}")]
    LoadError(String),
}

pub type Result<T> = std::result::Result<T, AssetError>;
