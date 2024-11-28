use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub screen_width: u32,
    pub screen_height: u32,
    pub vsync: bool,
    pub fov: f32,
    pub draw_distance: f32,
}
