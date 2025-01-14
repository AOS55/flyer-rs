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
    println!("Running Physics Integrator System");
    // Get the timestep duration in seconds
    let dt = config.timestep;

    // Iterate over all entities with physics and spatial components
    for (physics, mut spatial) in query.iter_mut() {
        println!("Integrator, physics: {:?}, spatial: {:?}", physics, spatial);
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
