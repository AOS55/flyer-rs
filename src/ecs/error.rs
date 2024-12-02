use crate::ecs::entity::EntityId;
use crate::resources::ResourceError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum EcsError {
    InvalidEntity(EntityId),
    ResourceError(ResourceError),
    ComponentError(String),
    SystemError(String),
}

impl fmt::Display for EcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcsError::InvalidEntity(id) => write!(f, "Invalid entity: {:?}", id),
            EcsError::ResourceError(err) => write!(f, "Resource error: {}", err),
            EcsError::ComponentError(msg) => write!(f, "Component error: {}", msg),
            EcsError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl Error for EcsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EcsError::ResourceError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ResourceError> for EcsError {
    fn from(err: ResourceError) -> Self {
        EcsError::ResourceError(err)
    }
}

pub type Result<T> = std::result::Result<T, EcsError>;
