use bevy::prelude::*;
use std::f64::consts::PI;

/// Configuration for the trim solver
#[derive(Resource, Debug, Clone, Copy)]
pub struct TrimSolverConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Convergence tolerance for cost function
    pub cost_tolerance: f64,
    /// Whether to use gradient refinement
    pub use_gradient_refinement: bool,
    /// Bounds for lateral control surfaces and states
    pub lateral_bounds: LateralBounds,
    /// Bounds for longitudinal control surfaces and states
    pub longitudinal_bounds: LongitudinalBounds,
}

#[derive(Debug, Clone, Copy)]
pub struct LongitudinalBounds {
    pub elevator_range: (f64, f64),
    pub throttle_range: (f64, f64),
    pub alpha_range: (f64, f64),
    pub theta_range: (f64, f64),
}

impl Default for LongitudinalBounds {
    fn default() -> Self {
        Self {
            elevator_range: (-1.0, 1.0),
            throttle_range: (0.0, 1.0),
            alpha_range: (-20.0 * PI / 180.0, 20.0 * PI / 180.0),
            theta_range: (-1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LateralBounds {
    pub aileron_range: (f64, f64),
    pub rudder_range: (f64, f64),
    pub beta_range: (f64, f64),
    pub phi_range: (f64, f64),
}

impl Default for LateralBounds {
    fn default() -> Self {
        Self {
            aileron_range: (-1.0, 1.0),
            rudder_range: (-1.0, 1.0),
            beta_range: (-20.0 * PI / 180.0, 20.0 * PI / 180.0),
            phi_range: (-80.0 * PI / 180.0, 80.0 * PI / 180.0),
        }
    }
}

pub struct TrimBounds {
    pub lateral_bounds: LateralBounds,
    pub longitudinal_bounds: LongitudinalBounds,
}

impl Default for TrimBounds {
    fn default() -> Self {
        Self {
            lateral_bounds: LateralBounds::default(),
            longitudinal_bounds: LongitudinalBounds::default(),
        }
    }
}

impl Default for TrimSolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            cost_tolerance: 1e-4,
            use_gradient_refinement: true,
            lateral_bounds: LateralBounds::default(),
            longitudinal_bounds: LongitudinalBounds::default(),
        }
    }
}
