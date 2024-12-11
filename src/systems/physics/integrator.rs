use crate::components::{PhysicsComponent, SpatialComponent};
use crate::resources::PhysicsConfig;
use bevy::prelude::*;
use nalgebra::UnitQuaternion;

/// Physics integration system that updates spatial state based on forces
pub fn physics_integrator_system(
    mut query: Query<(&PhysicsComponent, &mut SpatialComponent)>,
    time: Res<Time<Fixed>>,
    config: Res<PhysicsConfig>,
) {
    let dt = time.delta_secs_f64();

    for (physics, mut spatial) in query.iter_mut() {
        integrate_state(physics, &mut spatial, dt, &config);
    }
}

/// Core integration logic
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
    }
}

/// Apply velocity and angular velocity limits
fn apply_velocity_limits(spatial: &mut SpatialComponent, config: &PhysicsConfig) {
    let velocity_norm = spatial.velocity.norm();
    if velocity_norm > config.max_velocity {
        spatial.velocity *= config.max_velocity / velocity_norm;
    }

    let angular_velocity_norm = spatial.angular_velocity.norm();
    if angular_velocity_norm > config.max_angular_velocity {
        spatial.angular_velocity *= config.max_angular_velocity / angular_velocity_norm;
    }
}
