use bevy::prelude::*;

#[derive(Resource)]
pub struct AerodynamicsConfig {
    pub min_airspeed_threshold: f64,
}
