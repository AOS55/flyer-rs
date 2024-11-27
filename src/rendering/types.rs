use glam::Vec2;
use std::collections::HashMap;
use tiny_skia::Pixmap;

#[derive(Debug, Clone)]
pub enum RenderType {
    World,
    Aircraft,
    AircraftFixed,
}

#[derive(Debug)]
pub struct RenderConfig {
    pub screen_dims: Vec2,
    pub scale: f32,
    pub render_type: RenderType,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            screen_dims: Vec2::new(1024.0, 1024.0),
            scale: 25.0,
            render_type: RenderType::World,
        }
    }
}

pub struct RenderState {
    pub origin: Vec2,
    pub canvas: Option<Pixmap>,
    pub asset_map: HashMap<String, Pixmap>,
}
