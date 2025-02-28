use nalgebra::{UnitQuaternion, Vector3};

use crate::components::{AirData, AircraftControlSurfaces, SpatialComponent};

/// Represents different types of trim conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrimCondition {
    /// Straight and Level flight at specific airspeed
    StraightAndLevel { airspeed: f64 },
    /// Steady Climb and descent
    SteadyClimb { airspeed: f64, gamma: f64 },
    /// Steady turn at constant altitude
    CoordinatedTurn { airspeed: f64, bank_angle: f64 },
}

/// Longitudinal trim state
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LongitudinalTrimState {
    pub elevator: f64,
    pub power_lever: f64,
    pub alpha: f64,
    pub theta: f64,
}

/// Lateral trim state
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LateralTrimState {
    pub aileron: f64,
    pub rudder: f64,
    pub beta: f64,
    pub phi: f64,
}

/// State variables that define the trim solution
#[derive(Debug, Clone, Copy)]
pub struct TrimState {
    pub longitudinal: LongitudinalTrimState,
    pub lateral: LateralTrimState,
}

impl Default for LongitudinalTrimState {
    fn default() -> Self {
        Self {
            elevator: 0.0,
            power_lever: 0.3,
            alpha: 0.05,
            theta: 0.05,
        }
    }
}

impl Default for LateralTrimState {
    fn default() -> Self {
        Self {
            aileron: 0.0,
            rudder: 0.0,
            beta: 0.0,
            phi: 0.0,
        }
    }
}

impl Default for TrimState {
    fn default() -> Self {
        Self {
            longitudinal: LongitudinalTrimState::default(),
            lateral: LateralTrimState::default(),
        }
    }
}

impl LongitudinalTrimState {
    pub fn to_vector(&self) -> Vec<f64> {
        vec![self.elevator, self.power_lever, self.alpha, self.theta]
    }

    pub fn from_vector(vec: &[f64]) -> Self {
        Self {
            elevator: vec[0],
            power_lever: vec[1],
            alpha: vec[2],
            theta: vec[3],
        }
    }
}

impl LateralTrimState {
    pub fn to_vector(&self) -> Vec<f64> {
        vec![self.aileron, self.rudder, self.beta, self.phi]
    }

    pub fn from_vector(vec: &[f64]) -> Self {
        Self {
            aileron: vec[0],
            rudder: vec[1],
            beta: vec[2],
            phi: vec[3],
        }
    }
}

impl TrimState {
    pub fn to_trim_state(
        spatial: &SpatialComponent,
        control_surfaces: &AircraftControlSurfaces,
        air_data: &AirData,
    ) -> Self {
        let (phi, theta, _) = spatial.attitude.euler_angles();

        Self {
            longitudinal: LongitudinalTrimState {
                elevator: control_surfaces.elevator,
                power_lever: control_surfaces.power_lever,
                alpha: air_data.alpha,
                theta,
            },
            lateral: LateralTrimState {
                aileron: control_surfaces.aileron,
                rudder: control_surfaces.rudder,
                beta: air_data.beta,
                phi,
            },
        }
    }

    pub fn apply_trim_state(
        self,
        control_surfaces: &mut AircraftControlSurfaces,
        air_data: &mut AirData,
        spatial: &mut SpatialComponent,
    ) {
        // Apply longitudinal states
        control_surfaces.elevator = self.longitudinal.elevator;
        control_surfaces.power_lever = self.longitudinal.power_lever;
        air_data.alpha = self.longitudinal.alpha;

        // Apply lateral states
        control_surfaces.aileron = self.lateral.aileron;
        control_surfaces.rudder = self.lateral.rudder;
        air_data.beta = self.lateral.beta;

        spatial.attitude = UnitQuaternion::from_euler_angles(
            self.lateral.phi,
            self.longitudinal.theta,
            0.0, // Yaw not considered in trim
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct LongitudinalResiduals {
    pub vertical_force: f64,   // Lift - Weight balance
    pub horizontal_force: f64, // Thrust - Drag balance
    pub pitch_moment: f64,     // Pitch equilibrium
    pub gamma_error: f64,      // Flight path angle error
}

#[derive(Debug, Clone, Default)]
pub struct LateralResiduals {
    pub side_force: f64,      // Lateral force balance
    pub roll_moment: f64,     // Roll equilibrium
    pub yaw_moment: f64,      // Yaw equilibrium
    pub turn_rate_error: f64, // Turn rate matching
}

#[derive(Debug, Clone)]
pub struct TrimResiduals {
    pub longitudinal: LongitudinalResiduals,
    pub lateral: LateralResiduals,
}

impl Default for TrimResiduals {
    fn default() -> Self {
        Self {
            longitudinal: LongitudinalResiduals::default(),
            lateral: LateralResiduals::default(),
        }
    }
}

/// Results from the trim calculation
#[derive(Debug, Clone)]
pub struct TrimResult {
    pub state: TrimState,
    pub converged: bool,
    pub cost: f64,
    pub iterations: usize,
    pub residuals: TrimResiduals,
}
