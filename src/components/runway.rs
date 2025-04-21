use bevy::prelude::*;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct RunwayComponent {
    /// Center position of the runway threshold (start) in NED frame [m]
    pub position: Vector3<f64>,
    /// Heading/orientation of the runway centerline in radians (clockwise from North)
    pub heading: f64,
    /// Width of the runway [m]
    pub width: f64,
    /// Length of the runway [m]
    pub length: f64,
}

impl Default for RunwayComponent {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            heading: 0.0,
            width: 15.0,
            length: 300.0,
        }
    }
}
