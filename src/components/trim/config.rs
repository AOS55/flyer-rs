use bevy::prelude::*;

/// Configuration for the trim solver
#[derive(Resource, Debug, Clone)]
pub struct TrimSolverConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Convergence tolerance for cost function
    pub cost_tolerance: f64,
    /// Convergence tolerance for state changes
    pub state_tolerance: f64,
    /// Whether to use gradient refinement
    pub use_gradient_refinement: bool,
    /// Bounds for control surfaces and states
    pub bounds: TrimBounds,
}

impl Default for TrimSolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            cost_tolerance: 1e-4,
            state_tolerance: 1e-6,
            use_gradient_refinement: true,
            bounds: TrimBounds::default(),
        }
    }
}

/// Bounds for trim variables
#[derive(Debug, Clone)]
pub struct TrimBounds {
    pub elevator_range: (f64, f64),
    pub aileron_range: (f64, f64),
    pub rudder_range: (f64, f64),
    pub throttle_range: (f64, f64),
    pub alpha_range: (f64, f64),
    pub beta_range: (f64, f64),
    pub phi_range: (f64, f64),
    pub theta_range: (f64, f64),
}

impl Default for TrimBounds {
    fn default() -> Self {
        use std::f64::consts::PI;
        Self {
            elevator_range: (-0.5, 0.5),
            aileron_range: (-0.5, 0.5),
            rudder_range: (-0.5, 0.5),
            throttle_range: (0.0, 1.0),
            alpha_range: (-20.0 * PI / 180.0, 20.0 * PI / 180.0),
            beta_range: (-20.0 * PI / 180.0, 20.0 * PI / 180.0),
            phi_range: (-80.0 * PI / 180.0, 80.0 * PI / 180.0),
            theta_range: (-30.0 * PI / 180.0, 30.0 * PI / 180.0),
        }
    }
}
