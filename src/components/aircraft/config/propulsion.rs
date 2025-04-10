use bevy::prelude::*;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

/// Configuration for an aircraft engine
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PowerplantConfig {
    /// Name of the powerplant
    pub name: String,
    /// Maximum thrust at sea level static conditions (N)
    pub max_thrust: f64,
    /// Minimum thrust (typically idle) at sea level (N)
    pub min_thrust: f64,
    /// Engine position relative to aircraft CG (m)
    pub position: Vector3<f64>,
    /// Thrust line orientation in body frame (rad)
    pub orientation: Vector3<f64>,
    /// Thrust specific fuel consumption at cruise (kg/N/s)
    pub tsfc: f64,
    /// Time constant for engine spool-up (s)
    pub spool_up_time: f64,
    /// Time constant for engine spool-down (s)
    pub spool_down_time: f64,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PropulsionConfig {
    /// Configurations for each engine
    pub engines: Vec<PowerplantConfig>,
}

impl Default for PowerplantConfig {
    fn default() -> Self {
        Self {
            name: "Generic Engine".to_string(),
            max_thrust: 20000.0,
            min_thrust: 1000.0,
            position: Vector3::new(0.0, 0.0, 0.0),
            orientation: Vector3::new(1.0, 0.0, 0.0), // Forward-pointing
            tsfc: 0.4 / 3600.0,                       // Convert from kg/N/hr to kg/N/s
            spool_up_time: 3.0,
            spool_down_time: 2.0,
        }
    }
}

impl PropulsionConfig {
    /// Creates a new single-engine configuration
    pub fn single_engine(engine: PowerplantConfig) -> Self {
        Self {
            engines: vec![engine],
        }
    }

    /// Creates a twin-engine configuration with symmetric placement
    pub fn twin_engine(
        engine: PowerplantConfig,
        y_offset: f64,
        x_offset: f64,
        z_offset: f64,
    ) -> Self {
        let left_engine = PowerplantConfig {
            position: Vector3::new(x_offset, -y_offset, z_offset),
            name: "Left Engine".to_string(),
            ..engine.clone()
        };
        let right_engine = PowerplantConfig {
            position: Vector3::new(x_offset, y_offset, z_offset),
            name: "Right Engine".to_string(),
            ..engine
        };
        Self {
            engines: vec![left_engine, right_engine],
        }
    }

    /// Creates common aircraft configurations
    pub fn twin_otter() -> Self {
        let base_engine = PowerplantConfig {
            name: "PT6A-27".to_string(),
            max_thrust: 12000.0,
            min_thrust: 600.0,
            tsfc: 0.35 / 3600.0,
            spool_up_time: 2.5,
            spool_down_time: 1.8,
            ..Default::default()
        };
        Self::twin_engine(base_engine, 5.0, -0.5, 0.3)
    }

    pub fn f4_phantom() -> Self {
        let base_engine = PowerplantConfig {
            name: "J79-GE-17".to_string(),
            max_thrust: 79800.0,
            min_thrust: 4000.0,
            tsfc: 0.8 / 3600.0,
            spool_up_time: 4.0,
            spool_down_time: 3.0,
            ..Default::default()
        };
        Self::twin_engine(base_engine, 1.0, -1.0, 0.0)
    }
    
    pub fn cessna_172() -> Self {
        let engine = PowerplantConfig {
            name: "Lycoming O-320".to_string(),
            max_thrust: 3000.0,      // N - Simplified single value
            min_thrust: 0.0,         // N
            tsfc: 0.3 / 3600.0,      // kg/(N·s)
            spool_up_time: 1.0,      // s
            spool_down_time: 1.0,    // s
            ..Default::default()
        };
        Self::single_engine(engine)  // Single engine config
    }
}
