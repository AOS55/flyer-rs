#![warn(clippy::all)]

use aerso::types::*;
use aerso::*;

// A turboprop representation of the aircraft's power-plant
#[allow(dead_code)]
pub struct PowerPlant {
    /// Name of the power-plant/engine
    name: String,
    /// Maximum shaft-power [W]
    shaft_power: f64,
    /// Maximum velocity [m/s]
    v_max: f64,
    /// Maximum efficiency
    efficiency: f64,
}

impl PowerPlant {
    /// Create a PT6 powerplant
    pub fn pt6() -> Self {
        Self {
            name: "PT6".to_string(),
            shaft_power: 2.0 * 1.12e6,
            v_max: 40.0,
            efficiency: 0.6,
        }
    }
}

/// Create the AeroEffect for the [PowerPlant] data-class to generate relevant aero forces and torques
impl AeroEffect for PowerPlant {
    fn get_effect(
        &self,
        _airstate: AirState,
        _rates: Vector3,
        input: &Vec<f64>,
    ) -> (Force, Torque) {
        let thrust = ((self.shaft_power * self.efficiency) / self.v_max) * input[2];
        (Force::body(thrust, 0.0, 0.0), Torque::body(0.0, 0.0, 0.0))
    }
}
