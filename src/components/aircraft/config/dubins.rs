use crate::components::RandomStartConfig;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for a Dubins Aircraft model.
/// This model is used for path planning and motion representation, often in scenarios
/// involving simplified aircraft dynamics with constraints on speed, turn rate,
/// and climb/descent rates.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DubinsAircraftConfig {
    /// Name of the aircraft, used to identify the configuration.
    pub name: String,
    /// The maximum allowed speed of the aircraft (m/s).
    pub max_speed: f64,
    /// The minimum allowed speed of the aircraft (m/s).
    pub min_speed: f64,
    /// The maximum acceleration of the aircraft (m/s²).
    pub acceleration: f64,
    /// The maximum allowable bank angle for turns (radians).
    pub max_bank_angle: f64,
    /// The maximum turn rate of the aircraft (radians per second).
    pub max_turn_rate: f64,
    /// The maximum climb rate of the aircraft (m/s).
    pub max_climb_rate: f64,
    /// The maximum descent rate of the aircraft (m/s).
    pub max_descent_rate: f64,
    /// Optional configuration for randomized starting positions.
    /// If set, the aircraft will start at a random position defined by this configuration.
    pub random_start_config: Option<RandomStartConfig>,
}

impl Default for DubinsAircraftConfig {
    /// Default values are chosen to represent a generic small aircraft.
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
            random_start_config: Some(RandomStartConfig::default()),
        }
    }
}

impl DubinsAircraftConfig {
    /// Creates a new `DubinsAircraftConfig` with specified parameters.
    ///
    /// # Arguments
    /// * `name` - The name of the aircraft.
    /// * `max_speed` - The maximum speed of the aircraft (m/s).
    /// * `min_speed` - The minimum speed of the aircraft (m/s).
    /// * `acceleration` - The maximum acceleration (m/s²).
    /// * `max_bank_angle` - The maximum allowable bank angle (radians).
    /// * `max_turn_rate` - The maximum turn rate (radians per second).
    /// * `max_climb_rate` - The maximum climb rate (m/s).
    /// * `max_descent_rate` - The maximum descent rate (m/s).
    /// * `random_start_config` - Optional configuration for randomized starting positions.
    ///
    /// # Returns
    /// A fully initialized `DubinsAircraftConfig` with the provided values.
    pub fn new(
        name: String,
        max_speed: f64,
        min_speed: f64,
        acceleration: f64,
        max_bank_angle: f64,
        max_turn_rate: f64,
        max_climb_rate: f64,
        max_descent_rate: f64,
        random_start_config: Option<RandomStartConfig>,
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
