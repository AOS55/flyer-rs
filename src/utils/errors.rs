use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Config error: {0}")]
    InvalidConfig(String),

    #[error("Physics error: {0}")]
    PhysicsError(String),

    #[error("Asset error: {0}")]
    AssetError(String),

    #[error("Invalid control input: {0}")]
    InvalidControl(String),

    #[error("Vehicle error: {0}")]
    VehicleError(String),

    #[error("State error: {0}")]
    StateError(String),

    #[error("World error: {0}")]
    WorldError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Render error: {0}")]
    RenderError(String),
}
