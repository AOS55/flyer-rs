use bevy::prelude::*;
use nalgebra::Vector3;

#[derive(Component, Debug)]
pub struct CameraComponent {
    pub target: Option<Entity>,
    pub offset: Vector3<f64>, // Offset from target [m]
}

impl Default for CameraComponent {
    fn default() -> Self {
        CameraComponent {
            target: None,
            offset: Vector3::new(0.0, 0.0, 500.0),
        }
    }
}
