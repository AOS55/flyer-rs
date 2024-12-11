use bevy::prelude::*;
use nalgebra::{Vector2, Vector3};

pub trait RenderScale {
    type RenderSpace;

    fn to_render(&self, scale: &RenderConfig) -> Self::RenderSpace;

    fn to_physics(render_space: &Self::RenderSpace, scale: &RenderConfig) -> Self;
}

#[derive(Resource, Clone, Debug)]
pub struct RenderConfig {
    /// Conversion factor from meters to pixels (pixels = meters * scale)
    scale: f64,
}

impl RenderConfig {
    pub fn new(scale: f64) -> Self {
        Self { scale }
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }
}

impl RenderScale for Vector3<f64> {
    type RenderSpace = Vec3;

    fn to_render(&self, scale: &RenderConfig) -> Vec3 {
        Vec3::new(
            (self.x * scale.scale) as f32,
            (self.y * scale.scale) as f32,
            (self.z * scale.scale) as f32,
        )
    }

    fn to_physics(render_space: &Vec3, scale: &RenderConfig) -> Self {
        Vector3::new(
            render_space.x as f64 / scale.scale,
            render_space.y as f64 / scale.scale,
            render_space.z as f64 / scale.scale,
        )
    }
}

impl RenderScale for Vector2<f64> {
    type RenderSpace = Vec2;

    fn to_render(&self, scale: &RenderConfig) -> Vec2 {
        Vec2::new((self.x * scale.scale) as f32, (self.y * scale.scale) as f32)
    }

    fn to_physics(render_space: &Self::RenderSpace, scale: &RenderConfig) -> Self {
        Vector2::new(
            render_space.x as f64 / scale.scale,
            render_space.y as f64 / scale.scale,
        )
    }
}

impl RenderScale for f64 {
    type RenderSpace = f32;

    fn to_render(&self, scale: &RenderConfig) -> f32 {
        (*self * scale.scale) as f32
    }

    fn to_physics(render_space: &f32, scale: &RenderConfig) -> Self {
        *render_space as f64 / scale.scale
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}
