use crate::physics::components::forces::ForceSystem;
use crate::physics::error::PhysicsError;
use nalgebra::{Matrix3, UnitQuaternion, Vector3};

/// Handles motion integration and kinematics calculations
pub struct MotionSystem {
    /// Mass of the body [kg]
    mass: f64,
    /// Inertia tensor [kg⋅m²]
    inertia: Matrix3<f64>,
    /// Inverse of inertia tensor [kg⋅m²]^-1
    inertia_inv: Matrix3<f64>,
    /// Linear acceleration [m/s²]
    acceleration: Vector3<f64>,
    /// Angular acceleration [rad/s²]
    angular_acceleration: Vector3<f64>,
}

impl MotionSystem {
    /// Create a new motion system with given mass and inertia
    pub fn new(mass: f64, inertia: Matrix3<f64>) -> Result<Self, PhysicsError> {
        // Validate mass
        if mass <= 0.0 {
            return Err(PhysicsError::InvalidParameter(
                "Mass must be positive".into(),
            ));
        }

        // Validate inertia matrix
        if !is_inertia_valid(&inertia) {
            return Err(PhysicsError::InvalidParameter(
                "Invalid inertia tensor".into(),
            ));
        }

        // Calculate inverse inertia
        let inertia_inv = match inertia.try_inverse() {
            Some(inv) => inv,
            None => {
                return Err(PhysicsError::ComputationError(
                    "Failed to invert inertia tensor".into(),
                ))
            }
        };

        Ok(Self {
            mass,
            inertia,
            inertia_inv,
            acceleration: Vector3::zeros(),
            angular_acceleration: Vector3::zeros(),
        })
    }

    /// Update accelerations based on current forces
    pub fn update_accelerations(&mut self, force_system: &ForceSystem) -> Result<(), PhysicsError> {
        // Compute linear acceleration from net force (F = ma)
        self.acceleration = force_system.net_force() / self.mass;

        // Compute angular acceleration from net moment (τ = Iα)
        self.angular_acceleration = self.inertia_inv * force_system.net_moment();

        Ok(())
    }

    /// Integrate motion for one timestep using semi-implicit Euler integration
    pub fn integrate(
        &self,
        position: &mut Vector3<f64>,
        velocity: &mut Vector3<f64>,
        attitude: &mut UnitQuaternion<f64>,
        angular_velocity: &mut Vector3<f64>,
        dt: f64,
    ) -> Result<(), PhysicsError> {
        // Validate timestep
        if dt <= 0.0 {
            return Err(PhysicsError::InvalidParameter(
                "Timestep must be positive".into(),
            ));
        }

        // Update velocities first (semi-implicit Euler)
        *velocity += self.acceleration * dt;
        *angular_velocity += self.angular_acceleration * dt;

        // Update position
        *position += *velocity * dt;

        // Update attitude quaternion
        let omega = UnitQuaternion::from_scaled_axis(*angular_velocity * dt);
        *attitude = omega * *attitude;

        Ok(())
    }

    /// Compute kinetic energy of the system
    pub fn kinetic_energy(&self, velocity: &Vector3<f64>, angular_velocity: &Vector3<f64>) -> f64 {
        // Translational kinetic energy: 1/2 * m * v^2
        let translational = 0.5 * self.mass * velocity.norm_squared();

        // Rotational kinetic energy: 1/2 * ω^T * I * ω
        let rotational = 0.5 * angular_velocity.transpose() * (self.inertia * angular_velocity);

        translational + rotational[0]
    }

    /// Get the current linear acceleration
    pub fn linear_acceleration(&self) -> Vector3<f64> {
        self.acceleration
    }

    /// Get the current angular acceleration
    pub fn angular_acceleration(&self) -> Vector3<f64> {
        self.angular_acceleration
    }

    /// Transform a vector from body frame to inertial frame
    pub fn body_to_inertial(
        &self,
        vec: Vector3<f64>,
        attitude: &UnitQuaternion<f64>,
    ) -> Vector3<f64> {
        attitude * vec
    }

    /// Transform a vector from inertial frame to body frame
    pub fn inertial_to_body(
        &self,
        vec: Vector3<f64>,
        attitude: &UnitQuaternion<f64>,
    ) -> Vector3<f64> {
        attitude.inverse() * vec
    }

    /// Calculate velocity at a point offset from center of mass
    pub fn velocity_at_point(
        &self,
        center_velocity: &Vector3<f64>,
        angular_velocity: &Vector3<f64>,
        offset: &Vector3<f64>,
    ) -> Vector3<f64> {
        center_velocity + angular_velocity.cross(offset)
    }
}

/// Check if inertia tensor is valid (symmetric and positive definite)
fn is_inertia_valid(inertia: &Matrix3<f64>) -> bool {
    // Check symmetry
    if !is_matrix_symmetric(inertia) {
        return false;
    }

    // Check positive definiteness through eigenvalues
    let eigenvals = match inertia.symmetric_eigen().eigenvalues.as_slice() {
        [x, y, z] => [*x, *y, *z],
        _ => return false,
    };

    eigenvals.iter().all(|&v| v > 0.0)
}

/// Check if matrix is symmetric
fn is_matrix_symmetric(mat: &Matrix3<f64>) -> bool {
    const EPSILON: f64 = 1e-10;
    for i in 0..3 {
        for j in 0..3 {
            if (mat[(i, j)] - mat[(j, i)]).abs() > EPSILON {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_motion_system_creation() {
        let mass = 1.0;
        let inertia = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

        let motion = MotionSystem::new(mass, inertia).unwrap();
        assert_eq!(motion.mass, mass);
    }

    #[test]
    fn test_invalid_mass() {
        let mass = -1.0;
        let inertia = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

        assert!(MotionSystem::new(mass, inertia).is_err());
    }

    #[test]
    fn test_integration() {
        let mass = 1.0;
        let inertia = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

        let motion = MotionSystem::new(mass, inertia).unwrap();
        let mut position = Vector3::new(0.0, 0.0, 0.0);
        let mut velocity = Vector3::new(1.0, 0.0, 0.0);
        let mut attitude = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), 0.0);
        let mut angular_velocity = Vector3::new(0.0, 0.0, PI);

        motion
            .integrate(
                &mut position,
                &mut velocity,
                &mut attitude,
                &mut angular_velocity,
                1.0,
            )
            .unwrap();

        assert!((position.x - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_kinetic_energy() {
        let mass = 2.0;
        let inertia = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

        let motion = MotionSystem::new(mass, inertia).unwrap();
        let velocity = Vector3::new(1.0, 0.0, 0.0);
        let angular_velocity = Vector3::new(0.0, 0.0, 1.0);

        let energy = motion.kinetic_energy(&velocity, &angular_velocity);
        // KE = 1/2 * m * v^2 + 1/2 * ω^T * I * ω = 1/2 * 2 * 1 + 1/2 * 1 = 1.5
        assert!((energy - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_frame_transformations() {
        let mass = 1.0;
        let inertia = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

        let motion = MotionSystem::new(mass, inertia).unwrap();
        let vec = Vector3::new(1.0, 0.0, 0.0);
        let attitude = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), PI / 2.0);

        let transformed = motion.body_to_inertial(vec, &attitude);
        assert!((transformed.y - 1.0).abs() < 1e-10);

        let back_transformed = motion.inertial_to_body(transformed, &attitude);
        assert!((back_transformed - vec).norm() < 1e-10);
    }
}
