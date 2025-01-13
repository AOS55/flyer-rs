use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    InvalidPhysicsModel(String),
    InvalidAircraftType(String),
    InvalidParameter { name: String, value: String },
    MissingRequired(String),
    ValidationError(String),
    PythonError(String),
    JsonError(String),
    InvalidObservationType(String),
    MissingObservationSpace,
    InvalidActionType(String),
    MissingActionSpace,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidPhysicsModel(msg) => write!(f, "Invalid physics model: {}", msg),
            ConfigError::InvalidAircraftType(msg) => write!(f, "Invalid aircraft type: {}", msg),
            ConfigError::InvalidParameter { name, value } => {
                write!(f, "Invalid parameter '{}' with value '{}'", name, value)
            }
            ConfigError::MissingRequired(name) => write!(f, "Missing required parameter: {}", name),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ConfigError::PythonError(msg) => write!(f, "Python error: {}", msg),
            ConfigError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            ConfigError::InvalidObservationType(msg) => {
                write!(f, "Invalid observation type: {}", msg)
            }
            ConfigError::MissingObservationSpace => write!(f, "Missing observation space"),
            ConfigError::InvalidActionType(msg) => {
                write!(f, "Invalid action type: {}", msg)
            }
            ConfigError::MissingActionSpace => write!(f, "Missing action space"),
        }
    }
}

impl std::error::Error for ConfigError {}
