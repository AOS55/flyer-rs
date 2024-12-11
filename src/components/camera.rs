use bevy::prelude::*;
use nalgebra::Vector3;

#[derive(Component, Debug)]
pub struct CameraComponent {
    pub target: Option<Entity>,
    pub offset: Vector3<f64>, // Offset from target [m]
    pub deadzone: f64,        // Deadzone around target [m]
    pub smoothing: f32,       // Smoothing factor [0, 1]
    pub locked: bool,         // Lock camera to target
}

impl Default for CameraComponent {
    fn default() -> Self {
        CameraComponent {
            target: None,
            offset: Vector3::new(0.0, 0.0, 500.0),
            deadzone: 50.0,
            smoothing: 0.0,
            locked: false,
        }
    }
}
