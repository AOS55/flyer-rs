use crate::components::{PhysicsComponent, SpatialComponent};
use crate::ecs::error::Result;
use crate::ecs::query::ComponentPair;
use crate::ecs::{System, World};
use nalgebra::UnitQuaternion;

pub struct PhysicsIntegrator {
    max_velocity: f64,
    max_angular_velocity: f64,
}

impl PhysicsIntegrator {
    pub fn new() -> Self {
        Self {
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        }
    }

    fn integrate_state(&self, physics: &PhysicsComponent, spatial: &mut SpatialComponent, dt: f64) {
        // Linear acceleration
        let acceleration = physics.net_force / physics.mass;

        // Angular acceleration in body frame
        let angular_acceleration = physics.inertia_inv * physics.net_moment;

        // Update velocities (semi-implicit Euler)
        spatial.velocity += acceleration * dt;
        spatial.angular_velocity += angular_acceleration * dt;

        // Apply velocity limits
        self.apply_velocity_limits(spatial);

        // Update position
        spatial.position += spatial.velocity * dt;

        // Update attitude quaternion
        if spatial.angular_velocity.norm() > 0.0 {
            let rotation = UnitQuaternion::from_scaled_axis(spatial.angular_velocity * dt);
            spatial.attitude = rotation * spatial.attitude;
            _ = spatial.attitude.normalize();
        }
    }

    fn apply_velocity_limits(&self, spatial: &mut SpatialComponent) {
        let velocity_norm = spatial.velocity.norm();
        if velocity_norm > self.max_velocity {
            spatial.velocity *= self.max_velocity / velocity_norm;
        }

        let angular_velocity_norm = spatial.angular_velocity.norm();
        if angular_velocity_norm > self.max_angular_velocity {
            spatial.angular_velocity *= self.max_angular_velocity / angular_velocity_norm;
        }
    }
}

impl System for PhysicsIntegrator {
    fn name(&self) -> &str {
        "Physics Integrator"
    }

    fn run(&self, world: &mut World) -> Result<()> {
        let dt = *world.get_resource::<f64>()?;

        for (_, (physics, spatial)) in
            world.query_mut::<ComponentPair<PhysicsComponent, SpatialComponent>>()
        {
            self.integrate_state(physics, spatial, dt);
        }

        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["Force Calculator"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nalgebra::{Matrix3, UnitQuaternion, Vector3};

    // Helper function to create a basic physics component
    fn create_test_physics() -> PhysicsComponent {
        PhysicsComponent::new(
            1.0,                 // mass
            Matrix3::identity(), // inertia
        )
    }

    // Helper function to create a basic spatial component
    fn create_test_spatial() -> SpatialComponent {
        SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        }
    }

    #[test]
    fn test_linear_motion() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Apply a constant force in X direction (F = ma -> a = F/m)
        physics.net_force = Vector3::new(10.0, 0.0, 0.0); // 10N force

        // Integrate for 1 second
        integrator.integrate_state(&physics, &mut spatial, 1.0);

        // With F=10N and m=1kg, expect v=10m/s and x=5m (x = 1/2*a*t^2)
        assert_relative_eq!(spatial.velocity.x, 10.0, epsilon = 1e-10);
        assert_relative_eq!(spatial.position.x, 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rotational_motion() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Apply constant torque around Z axis
        physics.net_moment = Vector3::new(0.0, 0.0, 1.0);

        // Integrate for 1 second
        integrator.integrate_state(&physics, &mut spatial, 1.0);

        // With unit inertia, expect angular velocity = torque * time
        assert_relative_eq!(spatial.angular_velocity.z, 1.0, epsilon = 1e-10);

        // Check rotation angle (θ = ω * t)
        let (roll, pitch, yaw) = spatial.attitude.euler_angles();
        assert_relative_eq!(yaw, 1.0, epsilon = 1e-10);
        assert_relative_eq!(roll, 0.0, epsilon = 1e-10);
        assert_relative_eq!(pitch, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_velocity_limits() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Apply large force to exceed velocity limit
        physics.net_force = Vector3::new(1000.0, 0.0, 0.0);

        // Integrate for 1 second
        integrator.integrate_state(&physics, &mut spatial, 1.0);

        // Velocity should be capped at max_velocity (200.0)
        assert_relative_eq!(spatial.velocity.norm(), 200.0, epsilon = 1e-10);
    }

    #[test]
    fn test_angular_velocity_limits() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Apply large torque to exceed angular velocity limit
        physics.net_moment = Vector3::new(0.0, 0.0, 100.0);

        // Integrate for 1 second
        integrator.integrate_state(&physics, &mut spatial, 1.0);

        // Angular velocity should be capped at max_angular_velocity (10.0)
        assert_relative_eq!(spatial.angular_velocity.norm(), 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_multiple_timesteps() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Constant force in X direction (F = ma -> a = F/m = 10 m/s²)
        physics.net_force = Vector3::new(10.0, 0.0, 0.0);

        // First 0.5s step:
        // Δv = at = 10 * 0.5 = 5 m/s
        // Δx = v * t = 5 * 0.5 = 2.5 m
        integrator.integrate_state(&physics, &mut spatial, 0.5);

        // Second 0.5s step:
        // Δv = at = 10 * 0.5 = 5 m/s (final v = 10 m/s)
        // Δx = v * t = 10 * 0.5 = 5.0 m
        integrator.integrate_state(&physics, &mut spatial, 0.5);

        // Final velocity should be 10 m/s
        assert_relative_eq!(spatial.velocity.x, 10.0, epsilon = 1e-10);

        // Total position should be 7.5 m (2.5 + 5.0)
        assert_relative_eq!(spatial.position.x, 7.5, epsilon = 1e-10);
    }

    #[test]
    fn test_combined_motion() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Apply both force and torque
        physics.net_force = Vector3::new(10.0, 0.0, 0.0);
        physics.net_moment = Vector3::new(0.0, 0.0, 1.0);

        // Integrate for 1 second
        integrator.integrate_state(&physics, &mut spatial, 1.0);

        // Check both linear and angular motion
        assert_relative_eq!(spatial.velocity.x, 10.0, epsilon = 1e-10);
        assert_relative_eq!(spatial.position.x, 10.0, epsilon = 1e-10);
        assert_relative_eq!(spatial.angular_velocity.z, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_quaternion_normalization() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Apply rotation around multiple axes
        physics.net_moment = Vector3::new(1.0, 1.0, 1.0);

        // Integrate for several steps
        for _ in 0..10 {
            integrator.integrate_state(&physics, &mut spatial, 0.1);

            // Verify quaternion remains normalized
            assert_relative_eq!(spatial.attitude.as_vector().norm(), 1.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_zero_dt() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Set initial state
        spatial.position = Vector3::new(1.0, 2.0, 3.0);
        spatial.velocity = Vector3::new(1.0, 1.0, 1.0);
        physics.net_force = Vector3::new(10.0, 10.0, 10.0);

        // Integrate with dt = 0
        integrator.integrate_state(&physics, &mut spatial, 0.0);

        // State should remain unchanged
        assert_relative_eq!(spatial.position.x, 1.0, epsilon = 1e-10);
        assert_relative_eq!(spatial.position.y, 2.0, epsilon = 1e-10);
        assert_relative_eq!(spatial.position.z, 3.0, epsilon = 1e-10);
        assert_relative_eq!(spatial.velocity.norm(), 3.0_f64.sqrt(), epsilon = 1e-10);
    }

    #[test]
    fn test_very_small_timestep() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();
        let mut physics = create_test_physics();

        // Apply constant force
        physics.net_force = Vector3::new(10.0, 0.0, 0.0);

        // Integrate with very small timestep
        let dt = 1e-6;
        integrator.integrate_state(&physics, &mut spatial, dt);

        // Check for expected values
        let expected_velocity = 10.0 * dt;
        let expected_position = 0.5 * 10.0 * dt * dt;

        assert_relative_eq!(spatial.velocity.x, expected_velocity, epsilon = 1e-10);
        assert_relative_eq!(spatial.position.x, expected_position, epsilon = 1e-10);
    }

    #[test]
    fn test_inertia_tensor() {
        let integrator = PhysicsIntegrator::new();
        let mut spatial = create_test_spatial();

        // Create physics component with non-identity inertia tensor
        let inertia = Matrix3::new(2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0);
        let mut physics = PhysicsComponent::new(1.0, inertia);

        // Apply same torque to each axis
        physics.net_moment = Vector3::new(1.0, 1.0, 1.0);

        // Integrate for 1 second
        integrator.integrate_state(&physics, &mut spatial, 1.0);

        // Angular velocities should be inversely proportional to moments of inertia
        assert_relative_eq!(spatial.angular_velocity.x, 0.5, epsilon = 1e-10); // 1/2
        assert_relative_eq!(spatial.angular_velocity.y, 1.0 / 3.0, epsilon = 1e-10); // 1/3
        assert_relative_eq!(spatial.angular_velocity.z, 0.25, epsilon = 1e-10); // 1/4
    }
}
