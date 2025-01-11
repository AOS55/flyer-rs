use flyer::{
    components::{Force, ForceCategory, Moment, PhysicsComponent, ReferenceFrame},
    resources::PhysicsConfig,
};
use nalgebra::{Matrix3, Vector3};

pub mod fixtures {
    use super::*;

    pub static TEST_PHYSICS_CONFIG: PhysicsConfig = PhysicsConfig {
        max_velocity: 200.0,
        max_angular_velocity: 10.0,
        timestep: 1.0 / 120.0,
        gravity: Vector3::new(0.0, 0.0, 9.81),
    };
}

/// Creates a physics component with standard test mass and inertia
pub fn create_test_physics() -> PhysicsComponent {
    PhysicsComponent {
        mass: 1000.0,
        inertia: Matrix3::from_diagonal(&Vector3::new(1000.0, 2000.0, 1500.0)),
        inertia_inv: Matrix3::from_diagonal(&Vector3::new(0.001, 0.0005, 0.00067)),
        net_force: Vector3::zeros(),
        net_moment: Vector3::zeros(),
        forces: Vec::new(),
        moments: Vec::new(),
    }
}

/// Creates standard test forces for typical flight conditions
pub mod forces {
    use super::*;

    pub fn lift_force(magnitude: f64) -> Force {
        Force {
            vector: Vector3::new(0.0, 0.0, -magnitude),
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        }
    }

    pub fn drag_force(magnitude: f64) -> Force {
        Force {
            vector: Vector3::new(-magnitude, 0.0, 0.0),
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        }
    }

    pub fn thrust_force(magnitude: f64) -> Force {
        Force {
            vector: Vector3::new(magnitude, 0.0, 0.0),
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Propulsive,
        }
    }

    pub fn side_force(magnitude: f64) -> Force {
        Force {
            vector: Vector3::new(0.0, magnitude, 0.0),
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        }
    }
}

/// Creates standard test moments for typical flight conditions
pub mod moments {
    use super::*;

    pub fn roll_moment(magnitude: f64) -> Moment {
        Moment {
            vector: Vector3::new(magnitude, 0.0, 0.0),
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        }
    }

    pub fn pitch_moment(magnitude: f64) -> Moment {
        Moment {
            vector: Vector3::new(0.0, magnitude, 0.0),
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        }
    }

    pub fn yaw_moment(magnitude: f64) -> Moment {
        Moment {
            vector: Vector3::new(0.0, 0.0, magnitude),
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        }
    }
}

/// Physics configurations for different test scenarios
pub mod physics_configs {
    use super::*;

    pub fn basic_config() -> PhysicsConfig {
        PhysicsConfig {
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
            timestep: 1.0 / 120.0,
            gravity: Vector3::new(0.0, 0.0, 9.81),
        }
    }

    pub fn high_fidelity_config() -> PhysicsConfig {
        PhysicsConfig {
            max_velocity: 1000.0,
            max_angular_velocity: 20.0,
            timestep: 1.0 / 240.0,
            gravity: Vector3::new(0.0, 0.0, 9.81),
        }
    }
}
