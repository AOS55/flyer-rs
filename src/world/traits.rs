use crate::environment::{Runway, Terrain};
use crate::vehicles::traits::Vehicle;
use nalgebra::Vector3;

pub trait World {
    fn step(&mut self, dt: f64);
    fn reset(&mut self);
    fn add_vehicle(&mut self, vehicle: Box<dyn Vehicle>);
    fn update_vehicle(&mut self, vehicle: Box<dyn Vehicle>, id: usize);
    fn get_terrain(&self) -> &Terrain;
    fn get_camera_position(&self) -> Vector3<f64>;
    fn set_camera_position(&mut self, position: Vector3<f64>);
    fn set_goal(&mut self, position: Vector3<f64>);
    fn render(&mut self) -> tiny_skia::Pixmap;
}

pub trait WorldSettings {
    fn simulation_frequency(&self) -> f64;
    fn policy_frequency(&self) -> f64;
    fn render_frequency(&self) -> f64;
}
