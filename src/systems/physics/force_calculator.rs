use crate::components::{PhysicsComponent, ReferenceFrame, SpatialComponent};
use crate::config::physics::PhysicsConfig;
use bevy::prelude::*;
use nalgebra::Vector3;

pub fn force_calculator_system(
    mut query: Query<(&mut PhysicsComponent, &SpatialComponent)>,
    config: Res<PhysicsConfig>,
) {
    for (mut physics, spatial) in query.iter_mut() {
        // Reset net forces and moments
        physics.net_force = Vector3::zeros();
        physics.net_moment = Vector3::zeros();

        // Add gravitational force
        let gravity_force = config.gravity * physics.mass;
        physics.net_force += gravity_force;

        // Process forces (existing logic)
        for force in &physics.forces {
            let force_inertial = match force.frame {
                ReferenceFrame::Body => spatial.attitude * force.vector,
                ReferenceFrame::Inertial => force.vector,
                ReferenceFrame::Wind => spatial.attitude * force.vector,
            };

            physics.net_force += force_inertial;

            if let Some(point) = force.point {
                let point_inertial = spatial.attitude * point;
                let moment = point_inertial.cross(&force_inertial);
                physics.net_moment += moment;
            }
        }

        // Process moments (existing logic)
        for moment in &physics.moments {
            let moment_inertial = match moment.frame {
                ReferenceFrame::Body => spatial.attitude * moment.vector,
                ReferenceFrame::Inertial => moment.vector,
                ReferenceFrame::Wind => spatial.attitude * moment.vector,
            };
            physics.net_moment += moment_inertial;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Moment;
    use crate::components::{Force, ForceCategory};
    use approx::assert_relative_eq;
    use nalgebra::{UnitQuaternion, Vector3};

    // Helper function to create a basic physics component
    fn create_test_physics() -> PhysicsComponent {
        PhysicsComponent::new(
            1.0,                           // mass
            nalgebra::Matrix3::identity(), // inertia
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
    fn test_gravity_force() {}

    #[test]
    fn test_body_force_transformation() {}

    #[test]
    fn test_force_with_moment() {}

    #[test]
    fn test_multiple_forces() {}

    #[test]
    fn test_force_categories() {}

    #[test]
    fn test_moment_accumulation() {}

    #[test]
    fn test_force_clearing() {}
}
