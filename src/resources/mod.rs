mod assets;
mod config;
mod environment;
mod errors;
mod system;
mod time;

pub use assets::{AssetError, AssetManager, AssetType};
pub use config::environment::{
    AtmosphereConfig, AtmosphereType, EnvironmentConfig, WindModelConfig,
};
pub use config::render::RenderConfig;
pub use config::SimulationConfig;
pub use environment::EnvironmentResource;
pub use errors::{ResourceError, Result};
pub use system::ResourceSystem;
pub use time::TimeManager;

use std::any::Any;

pub trait Resource: Any + Send + Sync {}
impl<T: Any + Send + Sync> Resource for T {}
