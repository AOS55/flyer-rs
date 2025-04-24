use crate::components::{AirData, AircraftControlSurfaces, SpatialComponent};
use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// Represents different types of trim conditions
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
            elevator: 0.1,    // Start with slight positive elevator (based on analytical solution)
            power_lever: 0.7, // Higher power for 70 m/s cruise
            alpha: 0.15,      // ~8.6 degrees, based on analysis for 70 m/s
            theta: 0.15,      // Match alpha for level flight
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
        // CLAMP ALL VALUES before applying to ensure physically valid values
        // This is critical - without this, extreme optimizer values can propagate to the aircraft

        // Clamp longitudinal values
        let clamped_longitudinal = LongitudinalTrimState {
            // Elevator: -1.0 to 1.0
            elevator: self.longitudinal.elevator.clamp(-1.0, 1.0),
            // Power lever: 0.0 to 1.0
            power_lever: self.longitudinal.power_lever.clamp(0.0, 1.0),
            // Alpha: -10 to 20 degrees (-0.17 to 0.35 radians)
            alpha: self.longitudinal.alpha.clamp(-0.17, 0.35),
            // Theta: -10 to 20 degrees (-0.17 to 0.35 radians)
            theta: self.longitudinal.theta.clamp(-0.17, 0.35),
        };

        // Clamp lateral values
        let clamped_lateral = LateralTrimState {
            // Aileron: -1.0 to 1.0
            aileron: self.lateral.aileron.clamp(-1.0, 1.0),
            // Rudder: -1.0 to 1.0
            rudder: self.lateral.rudder.clamp(-1.0, 1.0),
            // Beta (sideslip): -0.17 to 0.17 radians (±10 degrees)
            beta: self.lateral.beta.clamp(-0.17, 0.17),
            // Phi (bank angle): -0.78 to 0.78 radians (±45 degrees)
            phi: self.lateral.phi.clamp(-0.78, 0.78),
        };

        // Print debug info if values were clamped
        if clamped_longitudinal.elevator != self.longitudinal.elevator
            || clamped_longitudinal.power_lever != self.longitudinal.power_lever
            || clamped_longitudinal.alpha != self.longitudinal.alpha
            || clamped_longitudinal.theta != self.longitudinal.theta
        {
            warn!("WARNING: Clamping extreme optimizer values:");
            info!(
                "  Original: elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°",
                self.longitudinal.elevator,
                self.longitudinal.power_lever,
                self.longitudinal.alpha.to_degrees(),
                self.longitudinal.theta.to_degrees()
            );
            info!(
                "  Clamped: elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°",
                clamped_longitudinal.elevator,
                clamped_longitudinal.power_lever,
                clamped_longitudinal.alpha.to_degrees(),
                clamped_longitudinal.theta.to_degrees()
            );
        }

        // Apply CLAMPED longitudinal states
        control_surfaces.elevator = clamped_longitudinal.elevator;
        control_surfaces.power_lever = clamped_longitudinal.power_lever;
        air_data.alpha = clamped_longitudinal.alpha;

        // Apply CLAMPED lateral states
        control_surfaces.aileron = clamped_lateral.aileron;
        control_surfaces.rudder = clamped_lateral.rudder;
        air_data.beta = clamped_lateral.beta;

        // Set attitude from CLAMPED Euler angles
        spatial.attitude = UnitQuaternion::from_euler_angles(
            clamped_lateral.phi,
            clamped_longitudinal.theta,
            0.0, // Yaw not considered in trim
        );

        // Get the airspeed magnitude (preserve existing magnitude)
        let airspeed = spatial.velocity.norm();

        // For straight and level flight:
        // 1. First set up the velocity in the world frame
        // 2. Ensure proper alpha angle relative to the body x-axis

        // Account for the sign convention in the aerodynamics system
        // In the air_data system, alpha = atan2(vz, vx), which means positive alpha
        // when the nose is above the velocity vector

        // Use absolute alpha value since the sign convention is handled by the velocity setting
        let abs_alpha = clamped_longitudinal.alpha.abs();

        // Set correct flight path angle (theta - |alpha|) to match our alpha definition
        let flight_path_angle = clamped_longitudinal.theta - abs_alpha;

        // Construct velocity in world frame aligned to flight path
        let vel_world = Vector3::new(
            airspeed * flight_path_angle.cos(),
            0.0,                                 // No lateral velocity for longitudinal trim
            -airspeed * flight_path_angle.sin(), // Negative because z is down in world frame
        );

        // Set the velocity
        spatial.velocity = vel_world;

        // Verify the alpha angle by transforming velocity to body frame
        let vel_body = spatial.attitude.inverse() * spatial.velocity;
        let calculated_alpha = (vel_body.z).atan2(vel_body.x);

        // Print debug info for verification
        info!("DEBUG: Applying trim state - Target Alpha: {:.2}°, FPA: {:.2}°, Calculated Alpha: {:.2}°",
                 clamped_longitudinal.alpha.to_degrees(),
                 flight_path_angle.to_degrees(),
                 calculated_alpha.to_degrees());
        info!(
            "DEBUG: Velocity: [{:.1}, {:.1}, {:.1}], Attitude Pitch: {:.2}°",
            spatial.velocity.x,
            spatial.velocity.y,
            spatial.velocity.z,
            clamped_longitudinal.theta.to_degrees()
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
