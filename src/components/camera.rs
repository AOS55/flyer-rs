use bevy::math::Vec2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Component)]
pub struct FlightCamera {
    #[reflect(ignore)]
    pub target: Option<Vec2>,
    pub interpolation_factor: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    #[reflect(ignore)]
    pub bounds: Option<(Vec2, Vec2)>,
}

impl Default for FlightCamera {
    fn default() -> Self {
        Self {
            target: None,
            interpolation_factor: 0.1,
            zoom_speed: 0.1,
            min_zoom: 0.1,
            max_zoom: 10.0,
            bounds: None,
        }
    }
}
