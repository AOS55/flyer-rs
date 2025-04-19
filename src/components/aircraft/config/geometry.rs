use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for the geometry of an aircraft.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AircraftGeometry {
    /// The total wing area of the aircraft (m²).
    pub wing_area: f64,
    /// The wingspan of the aircraft (m).
    pub wing_span: f64,
    /// The mean aerodynamic chord of the aircraft (m).
    pub mac: f64,
}

impl AircraftGeometry {
    /// Creates a new `AircraftGeometry` instance with the specified parameters.
    ///
    /// # Arguments
    /// * `wing_area` - The total wing area of the aircraft (m²).
    /// * `wing_span` - The wing span of the aircraft (m).
    /// * `mac` - The mean aerodynamic chord (m).
    ///
    /// # Returns
    /// A new instance of `AircraftGeometry` initialized with the given values.
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

    pub fn f16c() -> Self {
        Self::new(27.87, 3.45, 9.14)
    }

    pub fn cessna_172() -> Self {
        Self::new(16.2, 11.0, 1.6) // 16.2 m² wing area, 11.0 m wingspan, 1.6 m MAC
    }
}
