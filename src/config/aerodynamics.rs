use bevy::prelude::*;

use crate::components::aerodynamics::{AeroCoefficients, AircraftGeometry};

#[derive(Resource)]
pub struct AerodynamicsConfig {
    pub min_airspeed_threshold: f64,
    pub default_geometry: AircraftGeometry,
    pub default_coefficients: AeroCoefficients,
}

impl Default for AerodynamicsConfig {
    fn default() -> Self {
        Self {
            min_airspeed_threshold: 1e-6,
            default_geometry: AircraftGeometry::default(),
            default_coefficients: AeroCoefficients::default(),
        }
    }
}
