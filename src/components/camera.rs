use crate::ecs::component::Component;
use glam::Vec2;
use std::any::Any;

pub struct CameraComponent {
    pub position: Vec2,               // Current position
    pub viewport: Vec2,               // Viewport dimensions
    pub zoom: f32,                    // Camera zoom level
    pub bounds: Option<(Vec2, Vec2)>, // Optional world bounds
    pub target: Option<Vec2>,         // Optional follow target
    pub interpolation_factor: f32,    // Smoothing factor for camera movement
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            viewport: Vec2::new(1920.0, 1080.0),
            zoom: 1.0,
            bounds: None,
            target: None,
            interpolation_factor: 0.1,
        }
    }
}

impl Component for CameraComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
