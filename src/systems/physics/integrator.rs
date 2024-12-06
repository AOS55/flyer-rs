use crate::{
    components::{PhysicsComponent, SpatialComponent},
    config::physics::PhysicsConfig,
};
use bevy::prelude::*;
use nalgebra::UnitQuaternion;

/// Physics integration system that updates spatial state based on forces
pub fn physics_integrator_system(
    mut query: Query<(&PhysicsComponent, &mut SpatialComponent)>,
    time: Res<Time<Fixed>>,
    config: Res<PhysicsConfig>,
) {
    let dt = time.delta_seconds_f64();

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
        spatial.attitude.normalize_mut();
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nalgebra::Matrix3;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<PhysicsConfig>()
            .insert_resource(Time::<Fixed>::from_seconds_f64(1.0 / 120.0));
        app
    }

    fn spawn_test_entity(app: &mut App) -> Entity {
        app.world
            .spawn((
                PhysicsComponent::new(1.0, Matrix3::identity()),
                SpatialComponent::default(),
            ))
            .id()
    }

    #[test]
    fn test_linear_motion() {}

    #[test]
    fn test_rotational_motion() {}

    #[test]
    fn test_velocity_limits() {}

    #[test]
    fn test_quaternion_normalization() {}

    #[test]
    fn test_zero_dt() {}
}
