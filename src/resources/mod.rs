mod config;
mod environment;
mod terrain;

pub use config::environment::{
    AtmosphereConfig, AtmosphereType, EnvironmentConfig, WindModelConfig,
};
pub use environment::EnvironmentResource;
