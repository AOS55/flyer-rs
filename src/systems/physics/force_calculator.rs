use crate::components::{PhysicsComponent, ReferenceFrame, SpatialComponent};
use crate::resources::PhysicsConfig;
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

        // Extract forces and moments to avoid conflicting borrows
        let forces = physics.forces.clone();
        let moments = physics.moments.clone();

        // Process forces
        for force in &forces {
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

        // Process moments
        for moment in &moments {
            let moment_inertial = match moment.frame {
                ReferenceFrame::Body => spatial.attitude * moment.vector,
                ReferenceFrame::Inertial => moment.vector,
                ReferenceFrame::Wind => spatial.attitude * moment.vector,
            };
            physics.net_moment += moment_inertial;
        }
    }
}
