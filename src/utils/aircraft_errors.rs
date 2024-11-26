use std::fmt;
use std::io;

#[derive(Debug)]
pub enum AircraftError {
    Io(io::Error),
    SerdeYaml(serde_yaml::Error),
    NotFound(String),
}

impl fmt::Display for AircraftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AircraftError::Io(err) => write!(f, "IO error: {}", err),
            AircraftError::SerdeYaml(err) => write!(f, "Serialization error: {}", err),
            AircraftError::NotFound(file) => write!(f, "File not found: {}", file),
        }
    }
}

impl std::error::Error for AircraftError {}

impl From<io::Error> for AircraftError {
    fn from(err: io::Error) -> AircraftError {
        AircraftError::Io(err)
    }
}

impl From<serde_yaml::Error> for AircraftError {
    fn from(err: serde_yaml::Error) -> AircraftError {
        AircraftError::SerdeYaml(err)
    }
}
