use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunwayConfig {
    pub name: String,
    pub asset: String,
    pub dimensions: Vec2, // width, length [m]
    pub heading: f32,     // [degrees]
    pub approach_config: ApproachConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproachConfig {
    pub touchdown_fraction: f32, // fraction of runway that is landable
    pub faf_distance: f32,       // distance from final approach fix to touchdown [m]
    pub iaf_distance: f32,       // distance from intermediate fix to touchdown [m]
    pub iaf_lateral: f32,        // distance from iaf to approach track [m]
    pub intercept_angle: f32,    // angle to intercept final approach track [rad]
}

impl Default for RunwayConfig {
    fn default() -> Self {
        Self {
            name: "runway".to_string(),
            asset: "runway".to_string(),
            dimensions: Vec2::new(25.0, 1000.0), // width, length
            heading: 0.0,
            approach_config: ApproachConfig::default(),
        }
    }
}

impl Default for ApproachConfig {
    fn default() -> Self {
        Self {
            touchdown_fraction: 0.8,
            faf_distance: 3000.0 * 5.0,
            iaf_distance: 2000.0 * 5.0,
            iaf_lateral: 2000.0 * 5.0,
            intercept_angle: 30.0 * std::f32::consts::PI / 180.0,
        }
    }
}
