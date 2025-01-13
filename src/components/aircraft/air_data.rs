use bevy::prelude::*;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

/// Represents aerodynamic data for the aircraft.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AirData {
    /// True airspeed of the aircraft (m/s).
    pub true_airspeed: f64,
    /// Angle of attack (α) in radians.
    pub alpha: f64,
    /// Sideslip angle (β) in radians.
    pub beta: f64,
    /// Dynamic pressure acting on the aircraft (Pa).
    pub dynamic_pressure: f64,
    /// Air density (kg/m³).
    pub density: f64,
    /// Relative velocity vector of the aircraft (m/s).
    pub relative_velocity: Vector3<f64>,
    /// Wind velocity vector (m/s).
    pub wind_velocity: Vector3<f64>,
}

impl Default for AirData {
    /// Provides a default state for zero airspeed and no wind.
    fn default() -> Self {
        Self {
            true_airspeed: 0.0,
            alpha: 0.0,
            beta: 0.0,
            dynamic_pressure: 0.0,
            density: 1.225,
            relative_velocity: Vector3::zeros(),
            wind_velocity: Vector3::zeros(),
        }
    }
}
