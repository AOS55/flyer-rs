use crate::utils::errors::SimError;
use crate::vehicles::traits::Controls;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftControls {
    // Primary flight controls
    pub aileron: f64,  // [-1, 1]
    pub elevator: f64, // [-1, 1]
    pub rudder: f64,   // [-1, 1]
    pub throttle: f64, // [0, 1]

    // Secondary controls
    pub flaps: f64, // [0, 1]
    pub gear: bool,
    pub brake: f64, // [0, 1]
}

impl Controls for AircraftControls {
    fn validate(&self) -> Result<(), SimError> {
        // Validate control surface deflections are within bounds
        if !(-1.0..=1.0).contains(&self.aileron) {
            return Err(SimError::InvalidControl("aileron out of bounds".into()));
        }
        if !(-1.0..=1.0).contains(&self.elevator) {
            return Err(SimError::InvalidControl("elevator out of bounds".into()));
        }
        if !(-1.0..=1.0).contains(&self.rudder) {
            return Err(SimError::InvalidControl("rudder out of bounds".into()));
        }
        if !(0.0..=1.0).contains(&self.throttle) {
            return Err(SimError::InvalidControl("throttle out of bounds".into()));
        }
        if !(0.0..=1.0).contains(&self.flaps) {
            return Err(SimError::InvalidControl("flaps out of bounds".into()));
        }
        if !(0.0..=1.0).contains(&self.brake) {
            return Err(SimError::InvalidControl("brake out of bounds".into()));
        }
        Ok(())
    }

    fn interpolate(&self, other: &Self, factor: f64) -> Self {
        Self {
            aileron: self.aileron + (other.aileron - self.aileron) * factor,
            elevator: self.elevator + (other.elevator - self.elevator) * factor,
            rudder: self.rudder + (other.rudder - self.rudder) * factor,
            throttle: self.throttle + (other.throttle - self.throttle) * factor,
            flaps: self.flaps + (other.flaps - self.flaps) * factor,
            gear: if factor > 0.5 { other.gear } else { self.gear },
            brake: self.brake + (other.brake - self.brake) * factor,
        }
    }
}

impl Default for AircraftControls {
    fn default() -> Self {
        Self {
            aileron: 0.0,
            elevator: 0.0,
            rudder: 0.0,
            throttle: 0.0,
            flaps: 0.0,
            gear: false,
            brake: 0.0,
        }
    }
}
