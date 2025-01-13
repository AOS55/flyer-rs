mod air_data;
mod config;
mod dubins;
mod full;
mod propulsion;
mod render;

pub use air_data::AirData;
pub use config::{
    AircraftAeroCoefficients, AircraftConfig, AircraftGeometry, AircraftSource, AircraftType,
    DubinsAircraftConfig, FixedStartConfig, FullAircraftConfig, MassModel, PowerplantConfig,
    PropulsionConfig, RandomHeadingConfig, RandomPosConfig, RandomSpeedConfig, RandomStartConfig,
    RawAircraftConfig, StartConfig,
};
pub use dubins::{DubinsAircraftControls, DubinsAircraftState};
pub use full::{AircraftControlSurfaces, FullAircraftState};
pub use propulsion::{PowerplantState, PropulsionState};
pub use render::{AircraftRenderState, Attitude};

#[derive(Debug, Clone, Copy)]
pub enum AircraftControls {
    Dubins(DubinsAircraftControls),
    Full(AircraftControlSurfaces),
}

#[derive(Debug, Clone)]
pub enum AircraftState {
    Dubins(DubinsAircraftState),
    Full(FullAircraftState),
}
