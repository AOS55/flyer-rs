mod aero_coef;
mod aircraft;
mod dubins;
mod geometry;
mod loader;
mod mass;
mod start;

pub use aero_coef::AircraftAeroCoefficients;
pub use aircraft::{AircraftSource, AircraftType, FullAircraftConfig};
pub use dubins::DubinsAircraftConfig;
pub use geometry::AircraftGeometry;
pub use loader::{ConfigError, RawAircraftConfig};
pub use mass::MassModel;
pub use start::RandomStartPosConfig;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AircraftConfig {
    Full(FullAircraftConfig),
    Dubins(DubinsAircraftConfig),
}
