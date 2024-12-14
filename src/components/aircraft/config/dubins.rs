use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::RandomStartPosConfig;

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DubinsAircraftConfig {
    pub name: String,
    pub max_speed: f64,
    pub min_speed: f64,
    pub acceleration: f64,
    pub max_bank_angle: f64, // radians
    pub max_turn_rate: f64,
    pub max_climb_rate: f64,
    pub max_descent_rate: f64,
    pub random_start_config: Option<RandomStartPosConfig>,
}

impl Default for DubinsAircraftConfig {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            max_speed: 200.0,
            min_speed: 40.0,
            acceleration: 10.0,
            max_bank_angle: 45.0 * std::f64::consts::PI / 180.0,
            max_turn_rate: 0.5,
            max_climb_rate: 5.0,
            max_descent_rate: 15.0,
            random_start_config: Some(RandomStartPosConfig::default()),
        }
    }
}

impl DubinsAircraftConfig {
    pub fn new(
        name: String,
        max_speed: f64,
        min_speed: f64,
        acceleration: f64,
        max_bank_angle: f64,
        max_turn_rate: f64,
        max_climb_rate: f64,
        max_descent_rate: f64,
        random_start_config: Option<RandomStartPosConfig>,
    ) -> Self {
        Self {
            name,
            max_speed,
            min_speed,
            acceleration,
            max_bank_angle,
            max_turn_rate,
            max_climb_rate,
            max_descent_rate,
            random_start_config,
        }
    }
}
