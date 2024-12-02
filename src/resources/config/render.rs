use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub screen_width: u32,
    pub screen_height: u32,
    pub vsync: bool,
    pub fov: f32,
    pub draw_distance: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            screen_width: 800,
            screen_height: 600,
            vsync: true,
            fov: 60.0,
            draw_distance: 1000.0,
        }
    }
}
