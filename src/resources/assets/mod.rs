mod loader;
mod manager;

pub use manager::{AssetManager, AssetType};

#[derive(Debug)]
pub enum AssetError {
    IoError(std::io::Error),
    InvalidFormat(String),
    NotFound(String),
}

impl std::error::Error for AssetError {}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::IoError(e) => write!(f, "IO error: {}", e),
            AssetError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            AssetError::NotFound(s) => write!(f, "Asset not found: {}", s),
        }
    }
}

impl From<std::io::Error> for AssetError {
    fn from(error: std::io::Error) -> Self {
        AssetError::IoError(error)
    }
}

pub type Result<T> = std::result::Result<T, AssetError>;
