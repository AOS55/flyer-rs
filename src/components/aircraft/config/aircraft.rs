use bevy::prelude::*;
use core::fmt::Debug;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::components::aircraft::config::{ConfigError, RawAircraftConfig};
use crate::components::{AircraftAeroCoefficients, AircraftGeometry, MassModel};

#[derive(Component, Debug, Clone, Deserialize)]
pub struct FullAircraftConfig {
    pub name: String,
    pub ac_type: AircraftType,
    pub mass: MassModel,
    pub geometry: AircraftGeometry,
    pub aero_coef: AircraftAeroCoefficients,
}

impl Default for FullAircraftConfig {
    fn default() -> Self {
        Self {
            name: "TwinOtter".to_string(),
            ac_type: AircraftType::TwinOtter,
            mass: MassModel::twin_otter(),
            geometry: AircraftGeometry::twin_otter(),
            aero_coef: AircraftAeroCoefficients::twin_otter(),
        }
    }
}

impl FullAircraftConfig {
    pub fn new(source: AircraftSource) -> Result<Self, ConfigError> {
        match source {
            AircraftSource::Programmed(aircraft_type) => Ok(Self::from_programmed(aircraft_type)),
            AircraftSource::File(path) => Self::from_file(path),
        }
    }

    fn from_programmed(aircraft_type: AircraftType) -> Self {
        match aircraft_type {
            AircraftType::TwinOtter => Self {
                name: "TwinOtter".to_string(),
                ac_type: AircraftType::TwinOtter,
                mass: MassModel::twin_otter(),
                geometry: AircraftGeometry::twin_otter(),
                aero_coef: AircraftAeroCoefficients::twin_otter(),
            },
            AircraftType::F4Phantom => Self {
                name: "F4Phantom".to_string(),
                ac_type: AircraftType::F4Phantom,
                mass: MassModel::f4_phantom(),
                geometry: AircraftGeometry::f4_phantom(),
                aero_coef: AircraftAeroCoefficients::f4_phantom(),
            },
            AircraftType::GenericTransport => Self {
                name: "GenericTransport".to_string(),
                ac_type: AircraftType::GenericTransport,
                mass: MassModel::generic_transport(),
                geometry: AircraftGeometry::generic_transport(),
                aero_coef: AircraftAeroCoefficients::generic_transport(),
            },
            AircraftType::Custom(string) => Self {
                name: string.clone(),
                ac_type: AircraftType::Custom(string),
                mass: MassModel::twin_otter(),
                geometry: AircraftGeometry::twin_otter(),
                aero_coef: AircraftAeroCoefficients::twin_otter(),
            },
        }
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let file_contents = std::fs::read_to_string(path)?;
        let raw_config: RawAircraftConfig = serde_yaml::from_str(&file_contents)?;
        Self::from_raw_config(raw_config)
    }

    fn from_raw_config(raw: RawAircraftConfig) -> Result<Self, ConfigError> {
        // Convert the raw config into your structured config
        // This is where you'll map the flat YAML structure to your nested structs
        Ok(Self {
            name: raw.name.clone(),
            ac_type: AircraftType::Custom(raw.name.clone()),
            mass: MassModel::new(raw.mass, raw.ixx, raw.iyy, raw.izz, raw.ixz),
            geometry: AircraftGeometry::new(raw.wing_area, raw.wing_span, raw.mac),
            aero_coef: AircraftAeroCoefficients::from_raw(&raw)?,
        })
    }

    pub fn twin_otter() -> Self {
        Self::from_programmed(AircraftType::TwinOtter)
    }

    pub fn f4_phantom() -> Self {
        Self::from_programmed(AircraftType::F4Phantom)
    }

    pub fn generic_transport() -> Self {
        Self::from_programmed(AircraftType::GenericTransport)
    }
}

#[derive(Debug, Clone)]
pub enum AircraftSource {
    Programmed(AircraftType),
    File(PathBuf),
}

#[derive(Component, Debug, Clone, Deserialize, Hash, PartialEq, Eq)]
pub enum AircraftType {
    TwinOtter,
    F4Phantom,
    GenericTransport,
    Custom(String),
}

impl AircraftType {
    pub fn get_texture_path(&self) -> &str {
        match self {
            AircraftType::TwinOtter => "aircraft/twin_otter.png",
            AircraftType::F4Phantom => "aircraft/f4_phantom.png",
            AircraftType::GenericTransport => "aircraft/generic_transport.png",
            AircraftType::Custom(path) => path,
        }
    }
}
