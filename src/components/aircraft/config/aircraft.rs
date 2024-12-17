use bevy::prelude::*;
use core::fmt::Debug;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::components::aircraft::config::{ConfigError, RawAircraftConfig};
use crate::components::{AircraftAeroCoefficients, AircraftGeometry, MassModel};

/// The full aircraft configuration, including mass, geometry, and aerodynamic coefficients
#[derive(Component, Debug, Clone, Deserialize)]
pub struct FullAircraftConfig {
    /// Name of the aircraft, defaults to type name.
    pub name: String,
    /// Type of aircraft represnted as an enum e.g. TwinOtter, F4Phantom, etc.
    pub ac_type: AircraftType,
    /// Mass model of the aircraft, including weight and inertia properties.
    pub mass: MassModel,
    /// The geometric properties of the aircraft, such as wing span and fuselage dimensions.
    pub geometry: AircraftGeometry,
    /// Aerodynamic coefficients for calculating forces and moments on the aircraft.
    pub aero_coef: AircraftAeroCoefficients,
}

impl Default for FullAircraftConfig {
    /// The `TwinOtter` configuration is chosen as the default for convenience.
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
    /// Creates a new aircraft configuration from a given source.
    ///
    /// # Arguments
    /// * `source` - An `AircraftSource` enum specifying if the configuration is hardcoded
    ///              (`Programmed`) or loaded from a file (`File`).
    ///
    /// # Returns
    /// A `Result` containing the new configuration or an error if the file fails to load.
    pub fn new(source: AircraftSource) -> Result<Self, ConfigError> {
        match source {
            AircraftSource::Programmed(aircraft_type) => Ok(Self::from_programmed(aircraft_type)),
            AircraftSource::File(path) => Self::from_file(path),
        }
    }

    /// Creates an aircraft configuration for predefined (programmed) types.
    ///
    /// # Arguments
    /// * `aircraft_type` - The specific type of aircraft.
    ///
    /// # Returns
    /// A `FullAircraftConfig` initialized with hardcoded values.
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

    /// Creates an aircraft configuration by reading from a YAML file.
    ///
    /// # Arguments
    /// * `path` - Path to the YAML configuration file.
    ///
    /// # Returns
    /// A `Result` containing the loaded configuration or an error if deserialization fails.
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let file_contents = std::fs::read_to_string(path)?;
        let raw_config: RawAircraftConfig = serde_yaml::from_str(&file_contents)?;
        Self::from_raw_config(raw_config)
    }

    /// Converts a raw configuration into a structured aircraft configuration.
    ///
    /// # Arguments
    /// * `raw` - A `RawAircraftConfig` containing the deserialized fields.
    ///
    /// # Returns
    /// A `Result` containing the final structured configuration.
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

/// Source for aircraft configuration.
/// Can either be a hardcoded configuration (`Programmed`) or loaded from a file.
#[derive(Debug, Clone)]
pub enum AircraftSource {
    Programmed(AircraftType),
    File(PathBuf),
}

/// Enumeration of available aircraft types.
#[derive(Component, Debug, Clone, Deserialize, Hash, PartialEq, Eq)]
pub enum AircraftType {
    TwinOtter,
    F4Phantom,
    GenericTransport,
    Custom(String),
}

impl AircraftType {
    /// Retrieves the texture file path for the aircraft type.
    ///
    /// # Returns
    /// A string slice representing the file path to the texture.
    pub fn get_texture_path(&self) -> &str {
        match self {
            AircraftType::TwinOtter => "aircraft/twin_otter.png",
            AircraftType::F4Phantom => "aircraft/f4_phantom.png",
            AircraftType::GenericTransport => "aircraft/generic_transport.png",
            AircraftType::Custom(path) => path,
        }
    }
}
