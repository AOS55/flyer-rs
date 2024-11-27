use super::camera::Camera;
use crate::environment::{Runway, Terrain};
use crate::vehicles::traits::{Vehicle, VehicleConfig, VehicleControls, VehicleState};
use nalgebra::Vector3;
use std::collections::HashMap;

pub struct WorldState {
    pub vehicles: Vec<
        Box<dyn Vehicle<State = VehicleState, Controls = VehicleControls, Config = VehicleConfig>>,
    >,
    pub camera: Camera,
    pub terrain: Terrain,
    pub runway: Option<Runway>,
    pub goal: Option<Vector3<f64>>,
    pub render_type: String,
    pub position_log: Vec<Vector3<f64>>,
    pub screen_dimensions: (f32, f32),
    pub scale: f32,
    pub origin: Vector3<f64>,
}

impl WorldState {
    pub fn new(screen_dims: (f32, f32), scale: f32) -> Self {
        Self {
            vehicles: Vec::new(),
            camera: Camera::default(),
            terrain: Terrain::default(),
            runway: None,
            goal: None,
            render_type: "world".to_string(),
            position_log: Vec::new(),
            screen_dimensions: screen_dims,
            scale,
            origin: Vector3::zeros(),
        }
    }

    pub fn log_position(&mut self, position: Vector3<f64>) {
        self.position_log.push(position);
        if self.position_log.len() > 400 {
            self.position_log.remove(0);
        }
    }
}
