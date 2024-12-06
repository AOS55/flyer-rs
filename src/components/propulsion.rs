use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Different types of propulsion systems available
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PropulsionType {
    Piston,
    TurboProp,
    TurboJet,
    Electric,
}

/// Component containing propulsion system state data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropulsionComponent {
    /// Type of propulsion system
    pub propulsion_type: PropulsionType,

    /// Maximum available power [W]
    pub max_power: f64,

    /// Current throttle setting [0-1]
    pub throttle: f64,

    /// Overall efficiency [0-1]
    pub efficiency: f64,

    /// Current RPM
    pub rpm: f64,

    /// Maximum RPM
    pub max_rpm: f64,

    /// Engine temperature [K]
    pub temperature: f64,

    /// Fuel flow rate [kg/s]
    pub fuel_flow: f64,
}

impl Default for PropulsionComponent {
    fn default() -> Self {
        Self {
            propulsion_type: PropulsionType::TurboProp,
            max_power: 1000.0 * 1000.0, // 1000 kW
            throttle: 0.0,
            efficiency: 0.8,
            rpm: 0.0,
            max_rpm: 2100.0,
            temperature: 288.15, // 15°C in Kelvin
            fuel_flow: 0.0,
        }
    }
}
