mod aerodynamics;
mod aircraft;
mod environment;
mod physics;
// mod render;
mod agent;
mod rewards;
pub mod terrain;
mod transformations;

pub use aerodynamics::AerodynamicsConfig;
pub use aircraft::AircraftAssets;
pub use environment::{
    AtmosphereConfig, AtmosphereType, EnvironmentConfig, EnvironmentModel, WindConfig,
};
pub use physics::PhysicsConfig;
// pub use render::{RenderConfig, RenderScale};
pub use agent::{AgentConfig, AgentState, SimulationMode};
pub use rewards::RewardWeights;
pub use terrain::{
    BiomeConfig, BiomeThresholds, FeatureConfig, HeightNoiseConfig, MoistureNoiseConfig,
    NoiseConfig, RenderConfig, RiverNoiseConfig, TerrainConfig,
};
pub use transformations::{
    AttitudeTransform, Frame, PositionTransform, ScaleTransform, TransformError,
    TransformationBundle, TransformationResource, VelocityTransform,
};

// pub use config::environment::{
//     AtmosphereConfig, AtmosphereType, EnvironmentConfig, WindModelConfig,
// };
// pub use environment::EnvironmentResource;
