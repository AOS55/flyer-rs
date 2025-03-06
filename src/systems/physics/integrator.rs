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

    // Iterate over all entities with physics and spatial components
    for (physics, mut spatial) in query.iter_mut() {
        integrate_state(&physics, &mut spatial, dt, &config);
    }
}

/// Core integration logic to update spatial state based on physics forces and moments.
///
/// # Arguments
/// - `physics`: The `PhysicsComponent` containing forces, moments, and physical properties.
/// - `spatial`: The `SpatialComponent` to update position, velocity, and orientation.
/// - `dt`: The timestep duration (in seconds).
/// - `config`: The physics configuration resource for constraints and limits.
fn integrate_state(
    physics: &PhysicsComponent,
    spatial: &mut SpatialComponent,
    dt: f64,
    config: &PhysicsConfig,
) {
    // Calculate accelerations
    let acceleration = physics.net_force / physics.mass;
    let angular_acceleration = physics.inertia_inv * physics.net_moment;

    // Update velocities (semi-implicit Euler)
    spatial.velocity += acceleration * dt;
    spatial.angular_velocity += angular_acceleration * dt;

    // Apply velocity limits
    apply_velocity_limits(spatial, config);

    // Update position
    spatial.position += spatial.velocity * dt;

    // Update attitude quaternion
    if spatial.angular_velocity.norm() > 0.0 {
        let rotation = UnitQuaternion::from_scaled_axis(spatial.angular_velocity * dt);
        spatial.attitude = rotation * spatial.attitude;
    };
}

/// Applies velocity and angular velocity limits to prevent excessive motion.
///
/// # Arguments
/// - `spatial`: The `SpatialComponent` containing velocity and angular velocity.
/// - `config`: The physics configuration resource containing max limits.
fn apply_velocity_limits(spatial: &mut SpatialComponent, config: &PhysicsConfig) {
    // Limit linear velocity magnitude
    let velocity_norm = spatial.velocity.norm();
    if velocity_norm > config.max_velocity {
        spatial.velocity *= config.max_velocity / velocity_norm;
    }

    // Limit angular velocity magnitude
    let angular_velocity_norm = spatial.angular_velocity.norm();
    if angular_velocity_norm > config.max_angular_velocity {
        spatial.angular_velocity *= config.max_angular_velocity / angular_velocity_norm;
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
            integrate_state(&physics, &mut spatial, config.timestep, &config);
            
            // Calculate current energy
            let current_height = -spatial.position.z;
            let current_speed = spatial.velocity.norm();
            let current_ke = 0.5 * mass * current_speed * current_speed;
            let current_pe = mass * gravity.z * current_height;
            let current_total_energy = current_ke + current_pe;
            
            // Verify energy is conserved (within numerical tolerance)
            let energy_error = (current_total_energy - initial_total_energy).abs() / initial_total_energy;
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
        let expected_attitude = UnitQuaternion::from_euler_angles(expected_roll, 0.0, 0.0);
        
        // Run simulation
        for _ in 0..steps {
            integrate_state(&physics, &mut spatial, config.timestep, &config);
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
            integrate_state(&physics, &mut spatial, config.timestep, &config);
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
            let config = PhysicsConfig {
                timestep: dt,
                gravity: Vector3::new(0.0, 0.0, 9.81),
                max_velocity: 1000.0,
                max_angular_velocity: 100.0,
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
                integrate_state(&physics, &mut spatial, dt, &config);
                
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
                assert_relative_eq!(
                    quat_norm, 
                    1.0, 
                    epsilon = 1e-10,
                    max_relative = 1e-10
                );
            }
            
            println!("Numerical stability test passed for timestep = {}", dt);
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
            integrate_state(&physics, &mut spatial, config.timestep, &config);
            
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
            integrate_state(&physics, &mut spatial, config.timestep, &config);
            
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
