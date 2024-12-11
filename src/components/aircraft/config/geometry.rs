use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AircraftGeometry {
    pub wing_area: f64,
    pub wing_span: f64,
    pub mac: f64,
}

impl AircraftGeometry {
    pub fn new(wing_area: f64, wing_span: f64, mac: f64) -> Self {
        AircraftGeometry {
            wing_area,
            wing_span,
            mac,
        }
    }

    pub fn twin_otter() -> Self {
        Self::new(39.0, 19.8, 1.98)
    }

    pub fn f4_phantom() -> Self {
        Self::new(49.239, 11.787, 4.889)
    }

    pub fn generic_transport() -> Self {
        Self::new(0.548, 2.08, 0.279)
    }
}
