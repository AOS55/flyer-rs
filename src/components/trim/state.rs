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

/// State variables that define the trim solution
#[derive(Debug, Clone, Copy)]
pub struct TrimState {
    // Control Surface positions
    pub elevator: f64,
    pub aileron: f64,
    pub rudder: f64,
    pub power_lever: f64,

    // Aircraft state
    pub alpha: f64, // Angle of attack
    pub beta: f64,  // Sideslip angle
    pub phi: f64,   // Roll angle
    pub theta: f64, // Pitch angle
}

impl Default for TrimState {
    fn default() -> Self {
        Self {
            elevator: 0.0,
            aileron: 0.0,
            rudder: 0.0,
            power_lever: 0.3, // Reasonable initial guess
            alpha: 0.05,      // ~3 degrees
            beta: 0.0,
            phi: 0.0,
            theta: 0.05, // ~3 degrees
        }
    }
}

impl TrimState {
    /// Convert the trim state to a vector for optimization
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.elevator,
            self.aileron,
            self.rudder,
            self.power_lever,
            self.alpha,
            self.beta,
            self.phi,
            self.theta,
        ]
    }

    /// Creates a trim state from a vector
    pub fn from_vector(vec: &[f64]) -> Self {
        Self {
            elevator: vec[0],
            aileron: vec[1],
            rudder: vec[2],
            power_lever: vec[3],
            alpha: vec[4],
            beta: vec[5],
            phi: vec[6],
            theta: vec[7],
        }
    }

    pub fn to_trim_state(
        spatial: &SpatialComponent,
        control_surfaces: &AircraftControlSurfaces,
        air_data: &AirData,
    ) -> Self {
        let (phi, theta, _) = spatial.attitude.euler_angles();

        Self {
            elevator: control_surfaces.elevator,
            aileron: control_surfaces.aileron,
            rudder: control_surfaces.rudder,
            power_lever: control_surfaces.power_lever,
            alpha: air_data.alpha,
            beta: air_data.beta,
            phi,
            theta,
        }
    }

    pub fn apply_trim_state(
        self,
        control_surfaces: &mut AircraftControlSurfaces,
        air_data: &mut AirData,
        spatial: &mut SpatialComponent,
    ) {
        control_surfaces.aileron = self.aileron;
        control_surfaces.elevator = self.elevator;
        control_surfaces.rudder = self.rudder;
        control_surfaces.power_lever = self.power_lever;

        air_data.alpha = self.alpha;
        air_data.beta = self.beta;

        spatial.attitude = UnitQuaternion::from_euler_angles(
            self.phi, self.theta, 0.0, // Yaw not considered in trim
        );
    }
}

/// Results from the trim calculation
#[derive(Debug, Clone)]
pub struct TrimResult {
    /// The found trim state
    pub state: TrimState,
    /// Whether the solver converged
    pub converged: bool,
    /// Final cost value
    pub cost: f64,
    /// Number of iterations taken
    pub iterations: usize,
    /// Residual forces and moments
    pub residuals: TrimResiduals,
}

/// Residual forces and moments from trim calculation
#[derive(Debug, Clone, Default)]
pub struct TrimResiduals {
    /// Net forces in body frame (N)
    pub forces: Vector3<f64>,
    /// Net moments in body frame (N⋅m)
    pub moments: Vector3<f64>,
    /// Flight path angle error (rad)
    pub gamma_error: f64,
    /// Bank angle error (rad)
    pub mu_error: f64,
}
