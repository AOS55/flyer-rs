use bevy::prelude::*;

#[derive(Component)]
pub struct DubinsAircraftConfig {
    pub max_speed: f64,
    pub min_speed: f64,
    pub acceleration: f64,
    pub max_bank_angle: f64, // radians
    pub max_turn_rate: f64,
    pub max_climb_rate: f64,
    pub max_descent_rate: f64,
}

impl Default for DubinsAircraftConfig {
    fn default() -> Self {
        Self {
            max_speed: 200.0,
            min_speed: 40.0,
            acceleration: 10.0,
            max_bank_angle: 45.0 * std::f64::consts::PI / 180.0,
            max_turn_rate: 0.5,
            max_climb_rate: 5.0,
            max_descent_rate: 15.0,
        }
    }
}
