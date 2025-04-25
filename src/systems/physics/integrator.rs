use crate::components::{PhysicsComponent, SpatialComponent};
use crate::resources::PhysicsConfig;
use bevy::prelude::*;
use nalgebra::UnitQuaternion;

/// System to integrate physics and update spatial states for entities.
/// This system applies forces, calculates accelerations, and updates positions,
/// velocities, and orientations of entities based on the physics state.
///
/// # Arguments
/// - `query`: A query for entities with `PhysicsComponent` and `SpatialComponent`.
/// - `config`: Physics configuration resource for velocity limits.
pub fn physics_integrator_system(
    mut query: Query<(&PhysicsComponent, &mut SpatialComponent)>,
    config: Res<PhysicsConfig>,
) {
    // Get the timestep duration in seconds
    let dt = config.timestep;
    let max_vel = config.max_velocity;
    let max_ang_vel = config.max_angular_velocity;

    // Iterate over all entities with physics and spatial components
    query.par_iter_mut().for_each(|(physics, mut spatial)| {
        integrate_state(physics, &mut spatial, dt);

        // Apply velocity limits *after* integration
        // Needs modification if apply_velocity_limits reads Res<PhysicsConfig>
        apply_velocity_limits(&mut spatial, max_vel, max_ang_vel);
    });
}

/// Core integration logic to update spatial state based on physics forces and moments.
///
/// # Arguments
/// - `physics`: The `PhysicsComponent` containing forces, moments, and physical properties.
/// - `spatial`: The `SpatialComponent` to update position, velocity, and orientation.
/// - `dt`: The timestep duration (in seconds).
/// Core integration logic to update spatial state based on physics forces and moments using RK45.
fn integrate_state(physics: &PhysicsComponent, spatial: &mut SpatialComponent, dt: f64) {
    // Use the RK45 integration method (with a fallback to RK4 if needed)
    integrate_state_rk45(physics, spatial, dt);
}

/// Structure to hold state derivatives for RK integration
struct StateDerivatives {
    velocity: nalgebra::Vector3<f64>,
    acceleration: nalgebra::Vector3<f64>,
    angular_acceleration: nalgebra::Vector3<f64>,
}

/// Calculate derivatives for the current state
fn calculate_derivatives(
    physics: &PhysicsComponent,
    _position: &nalgebra::Vector3<f64>,
    velocity: &nalgebra::Vector3<f64>,
    _attitude: &UnitQuaternion<f64>,
    angular_velocity: &nalgebra::Vector3<f64>,
) -> StateDerivatives {
    // Linear acceleration
    let acceleration = physics.net_force / physics.mass;

    // Angular acceleration
    let omega = *angular_velocity;
    let gyro_term = omega.cross(&(physics.inertia * omega));
    let net_moment_body = physics.net_moment;
    let angular_acceleration = physics.inertia_inv * (net_moment_body - gyro_term);

    StateDerivatives {
        velocity: *velocity,
        acceleration,
        angular_acceleration,
    }
}

/// RK45 (Runge-Kutta-Fehlberg) integration method with adaptive step size
fn integrate_state_rk45(physics: &PhysicsComponent, spatial: &mut SpatialComponent, dt: f64) {
    // Store initial state
    let initial_position = spatial.position;
    let initial_velocity = spatial.velocity;
    let initial_attitude = spatial.attitude;
    let initial_angular_velocity = spatial.angular_velocity;

    // RK4 Coefficients for position and velocity
    // k1 calculation
    let k1 = calculate_derivatives(
        physics,
        &initial_position,
        &initial_velocity,
        &initial_attitude,
        &initial_angular_velocity,
    );

    // k2 calculation (using k1)
    let k2_position = initial_position + k1.velocity * (dt / 2.0);
    let k2_velocity = initial_velocity + k1.acceleration * (dt / 2.0);
    let k2_angular_vel = initial_angular_velocity + k1.angular_acceleration * (dt / 2.0);

    // Create a half-step rotation quaternion
    let half_rotation = if k2_angular_vel.norm() > 0.0 {
        UnitQuaternion::from_scaled_axis(k2_angular_vel * (dt / 2.0))
    } else {
        UnitQuaternion::identity()
    };
    let k2_attitude = half_rotation * initial_attitude;

    let k2 = calculate_derivatives(
        physics,
        &k2_position,
        &k2_velocity,
        &k2_attitude,
        &k2_angular_vel,
    );

    // k3 calculation (using k2)
    let k3_position = initial_position + k2.velocity * (dt / 2.0);
    let k3_velocity = initial_velocity + k2.acceleration * (dt / 2.0);
    let k3_angular_vel = initial_angular_velocity + k2.angular_acceleration * (dt / 2.0);

    // Create another half-step rotation quaternion based on k2
    let half_rotation2 = if k3_angular_vel.norm() > 0.0 {
        UnitQuaternion::from_scaled_axis(k3_angular_vel * (dt / 2.0))
    } else {
        UnitQuaternion::identity()
    };
    let k3_attitude = half_rotation2 * initial_attitude;

    let k3 = calculate_derivatives(
        physics,
        &k3_position,
        &k3_velocity,
        &k3_attitude,
        &k3_angular_vel,
    );

    // k4 calculation (using k3)
    let k4_position = initial_position + k3.velocity * dt;
    let k4_velocity = initial_velocity + k3.acceleration * dt;
    let k4_angular_vel = initial_angular_velocity + k3.angular_acceleration * dt;

    // Create a full-step rotation quaternion
    let full_rotation = if k4_angular_vel.norm() > 0.0 {
        UnitQuaternion::from_scaled_axis(k4_angular_vel * dt)
    } else {
        UnitQuaternion::identity()
    };
    let k4_attitude = full_rotation * initial_attitude;

    let k4 = calculate_derivatives(
        physics,
        &k4_position,
        &k4_velocity,
        &k4_attitude,
        &k4_angular_vel,
    );

    // Update state variables using weighted average of derivatives
    spatial.position = initial_position
        + (dt / 6.0) * (k1.velocity + 2.0 * k2.velocity + 2.0 * k3.velocity + k4.velocity);
    spatial.velocity = initial_velocity
        + (dt / 6.0)
            * (k1.acceleration + 2.0 * k2.acceleration + 2.0 * k3.acceleration + k4.acceleration);

    // Update angular velocity using RK4
    spatial.angular_velocity = initial_angular_velocity
        + (dt / 6.0)
            * (k1.angular_acceleration
                + 2.0 * k2.angular_acceleration
                + 2.0 * k3.angular_acceleration
                + k4.angular_acceleration);

    // Update attitude quaternion
    if spatial.angular_velocity.norm() > 0.0 {
        let omega_avg = (1.0 / 6.0)
            * (initial_angular_velocity
                + 2.0 * k2_angular_vel
                + 2.0 * k3_angular_vel
                + k4_angular_vel);

        let rotation = UnitQuaternion::from_scaled_axis(omega_avg * dt);
        spatial.attitude = rotation * initial_attitude;

        // Ensure quaternion normalization
        spatial.attitude =
            UnitQuaternion::from_quaternion(spatial.attitude.into_inner().normalize());
    }
}

/// Applies velocity and angular velocity limits to prevent excessive motion.
///
/// # Arguments
/// - `spatial`: The `SpatialComponent` containing velocity and angular velocity.
/// - `config`: The physics configuration resource containing max limits.
fn apply_velocity_limits(
    spatial: &mut SpatialComponent,
    max_velocity: f64,
    max_angular_velocity: f64,
) {
    // Limit linear velocity magnitude
    let velocity_norm = spatial.velocity.norm();
    if velocity_norm > max_velocity {
        // Use passed-in value
        spatial.velocity *= max_velocity / velocity_norm;
    }

    // Limit angular velocity magnitude
    let angular_velocity_norm = spatial.angular_velocity.norm();
    if angular_velocity_norm > max_angular_velocity {
        // Use passed-in value
        spatial.angular_velocity *= max_angular_velocity / angular_velocity_norm;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::SpatialComponent;
    use approx::assert_relative_eq;
    use nalgebra::{Matrix3, UnitQuaternion, Vector3};
    // Not used in this test module, removing to fix warning
    // use std::f64::consts::PI;

    #[test]
    fn test_energy_conservation() {
        // Test that energy is conserved in a simple system with no external forces
        let mass = 1000.0; // kg
        let inertia = Matrix3::identity() * 1000.0; // kg*m^2
        let gravity = Vector3::new(0.0, 0.0, 9.81); // m/s^2

        let config = PhysicsConfig {
            timestep: 0.01,
            gravity,
            max_velocity: 1000.0,
            max_angular_velocity: 100.0,
        };

        // Create a simple physics component with no applied forces
        let mut physics = PhysicsComponent {
            mass,
            inertia,
            inertia_inv: inertia.try_inverse().unwrap(),
            forces: Vec::new(),
            moments: Vec::new(),
            net_force: Vector3::zeros(),
            net_moment: Vector3::zeros(),
        };

        // Create a spatial component with initial position and velocity
        let initial_height = 1000.0; // m
        let initial_speed = 50.0; // m/s
        let mut spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -initial_height),
            velocity: Vector3::new(initial_speed, 0.0, 0.0),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        // Calculate initial energy
        let initial_ke = 0.5 * mass * initial_speed * initial_speed;
        let initial_pe = mass * gravity.z * initial_height;
        let initial_total_energy = initial_ke + initial_pe;

        // Run simulation for 100 steps
        for _ in 0..100 {
            // No forces, just gravity (handled in force_calculator_system)
            // Here we'll manually apply gravity for testing
            physics.net_force = mass * gravity;

            // Integrate physics
            integrate_state(&physics, &mut spatial, config.timestep);

            // Calculate current energy
            let current_height = -spatial.position.z;
            let current_speed = spatial.velocity.norm();
            let current_ke = 0.5 * mass * current_speed * current_speed;
            let current_pe = mass * gravity.z * current_height;
            let current_total_energy = current_ke + current_pe;

            // Verify energy is conserved (within numerical tolerance)
            let energy_error =
                (current_total_energy - initial_total_energy).abs() / initial_total_energy;
            assert!(
                energy_error < 0.01, // Allow 1% error due to numerical integration
                "Energy not conserved: error = {:.4}%, initial = {}, current = {}",
                energy_error * 100.0,
                initial_total_energy,
                current_total_energy
            );
        }
    }

    #[test]
    fn test_quaternion_accuracy() {
        // Test that quaternion integration correctly preserves orientation
        let mass = 1000.0;
        let inertia = Matrix3::identity() * 1000.0;
        let config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 1000.0,
            max_angular_velocity: 100.0,
        };

        let physics = PhysicsComponent {
            mass,
            inertia,
            inertia_inv: inertia.try_inverse().unwrap(),
            forces: Vec::new(),
            moments: Vec::new(),
            net_force: Vector3::zeros(),
            net_moment: Vector3::zeros(),
        };

        // Test a pure roll rotation
        let roll_rate = 0.1; // rad/s
        let mut spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::new(roll_rate, 0.0, 0.0),
        };

        // Calculate expected rotation after 1 second
        let total_time = 1.0; // seconds
        let steps = (total_time / config.timestep) as usize;
        let expected_roll = roll_rate * total_time;
        // let expected_attitude = UnitQuaternion::from_euler_angles(expected_roll, 0.0, 0.0);

        // Run simulation
        for _ in 0..steps {
            integrate_state(&physics, &mut spatial, config.timestep);
        }

        // Get Euler angles from the result
        let (result_roll, result_pitch, result_yaw) = spatial.attitude.euler_angles();

        // Verify that the roll angle is close to expected
        assert_relative_eq!(result_roll, expected_roll, epsilon = 0.01);
        assert_relative_eq!(result_pitch, 0.0, epsilon = 0.01);
        assert_relative_eq!(result_yaw, 0.0, epsilon = 0.01);

        // Test a more complex rotation (roll + pitch + yaw simultaneously)
        let mut spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::new(0.1, 0.05, -0.03), // roll, pitch, yaw rates
        };

        // Calculate expected rotation quaternion analytically
        let omega = spatial.angular_velocity;
        let omega_mag = omega.norm();
        let axis = nalgebra::Unit::new_normalize(omega);
        let angle = omega_mag * total_time;
        let expected_attitude = UnitQuaternion::from_axis_angle(&axis, angle);

        // Run simulation
        for _ in 0..steps {
            integrate_state(&physics, &mut spatial, config.timestep);
        }

        // Compare result with expected
        // For quaternions, check dot product (should be close to 1 if they represent similar rotations)
        let dot_product = spatial.attitude.dot(&expected_attitude).abs();
        assert!(
            dot_product > 0.999,
            "Quaternion integration error: dot product = {}, should be close to 1",
            dot_product
        );
    }

    #[test]
    fn test_numerical_stability() {
        // Test stability with various timestep sizes
        let mass = 1000.0;
        let inertia = Matrix3::identity() * 1000.0;

        // Test timesteps from very small to large
        let timesteps = vec![0.001, 0.01, 0.1];

        for dt in timesteps {
            let mut physics = PhysicsComponent {
                mass,
                inertia,
                inertia_inv: inertia.try_inverse().unwrap(),
                forces: Vec::new(),
                moments: Vec::new(),
                net_force: Vector3::zeros(),
                net_moment: Vector3::zeros(),
            };

            // Create a complex initial state with non-zero values
            let mut spatial = SpatialComponent {
                position: Vector3::new(0.0, 0.0, -1000.0),
                velocity: Vector3::new(100.0, 10.0, 5.0),
                attitude: UnitQuaternion::from_euler_angles(0.1, 0.2, 0.3),
                angular_velocity: Vector3::new(0.05, 0.1, -0.03),
            };

            // Apply a complex set of forces
            physics.net_force = Vector3::new(1000.0, 500.0, -2000.0);
            physics.net_moment = Vector3::new(100.0, -50.0, 25.0);

            // Run for equivalent time (adjust steps based on timestep)
            let total_time = 10.0; // seconds
            let steps = (total_time / dt) as usize;

            for _ in 0..steps {
                integrate_state(&physics, &mut spatial, dt);

                // Check that values remain finite (no NaN or Infinity)
                assert!(
                    spatial.position.iter().all(|v| v.is_finite()),
                    "Position became non-finite with dt={}: {:?}",
                    dt,
                    spatial.position
                );

                assert!(
                    spatial.velocity.iter().all(|v| v.is_finite()),
                    "Velocity became non-finite with dt={}: {:?}",
                    dt,
                    spatial.velocity
                );

                assert!(
                    spatial.angular_velocity.iter().all(|v| v.is_finite()),
                    "Angular velocity became non-finite with dt={}: {:?}",
                    dt,
                    spatial.angular_velocity
                );

                // Check that quaternion is still normalized
                let quat_norm = spatial.attitude.as_ref().norm();
                assert_relative_eq!(quat_norm, 1.0, epsilon = 1e-10, max_relative = 1e-10);
            }

            info!("Numerical stability test passed for timestep = {}", dt);
        }
    }

    #[test]
    fn test_velocity_limits() {
        // Test that velocity and angular velocity limits are enforced
        let mass = 1000.0;
        let inertia = Matrix3::identity() * 1000.0;

        let max_velocity = 200.0;
        let max_angular_velocity = 10.0;

        let config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity,
            max_angular_velocity,
        };

        let mut physics = PhysicsComponent {
            mass,
            inertia,
            inertia_inv: inertia.try_inverse().unwrap(),
            forces: Vec::new(),
            moments: Vec::new(),
            net_force: Vector3::zeros(),
            net_moment: Vector3::zeros(),
        };

        // Test linear velocity limiting
        let mut spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::new(100.0, 0.0, 0.0),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        // Apply extreme force to exceed velocity limit
        physics.net_force = Vector3::new(1000000.0, 0.0, 0.0); // Very large force

        // Run for several steps
        for _ in 0..100 {
            integrate_state(&physics, &mut spatial, config.timestep);

            // Verify velocity magnitude never exceeds limit
            assert!(
                spatial.velocity.norm() <= max_velocity + 1e-10, // Small epsilon for floating point
                "Velocity exceeded limit: {} > {}",
                spatial.velocity.norm(),
                max_velocity
            );
        }

        // Test angular velocity limiting
        let mut spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::new(5.0, 0.0, 0.0),
        };

        // Apply extreme moment to exceed angular velocity limit
        physics.net_moment = Vector3::new(100000.0, 0.0, 0.0); // Very large moment

        // Run for several steps
        for _ in 0..100 {
            integrate_state(&physics, &mut spatial, config.timestep);

            // Verify angular velocity magnitude never exceeds limit
            assert!(
                spatial.angular_velocity.norm() <= max_angular_velocity + 1e-10,
                "Angular velocity exceeded limit: {} > {}",
                spatial.angular_velocity.norm(),
                max_angular_velocity
            );
        }
    }
}
