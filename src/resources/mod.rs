mod aerodynamics;
mod aircraft;
mod environment;
mod physics;
mod render;
pub mod terrain;
mod transformations;

pub use aerodynamics::AerodynamicsConfig;
pub use aircraft::AircraftAssets;
pub use environment::{
    AtmosphereConfig, AtmosphereType, EnvironmentConfig, EnvironmentModel, WindConfig,
};
pub use physics::PhysicsConfig;
pub use render::{RenderConfig, RenderScale};
pub use transformations::{
    AttitudeTransform, Frame, PositionTransform, ScaleTransform, TransformationResource,
};

// pub use config::environment::{
//     AtmosphereConfig, AtmosphereType, EnvironmentConfig, WindModelConfig,
// };
// pub use environment::EnvironmentResource;
