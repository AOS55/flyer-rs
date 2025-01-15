use crate::components::{
    Force, ForceCategory, Moment, PhysicsComponent, ReferenceFrame, SpatialComponent,
};
use crate::resources::PhysicsConfig;
use bevy::prelude::*;

/// A system to calculate the net forces and moments acting on entities with a `PhysicsComponent`.
/// It processes:
/// - Gravitational force.
/// - Forces and moments applied in different reference frames (Body, Inertial, Wind).
/// The results are stored in the `net_force` and `net_moment` fields of the `PhysicsComponent`.
///
/// # Arguments
/// - `query`: Query to access entities with FullAircraftState.
/// - `config`: A resource containing global physics configuration parameters, like gravity.
pub fn force_calculator_system(
    mut query: Query<(&mut PhysicsComponent, &SpatialComponent)>,
    config: Res<PhysicsConfig>,
) {
    // println!("Running Force Calculator System!");
    for (mut physics, spatial) in query.iter_mut() {
        // println!("physics: {:?}, spatial, {:?}", physics, config);

        physics
            .forces
            .retain(|force| force.category != ForceCategory::Gravitational);

        physics
            .moments
            .retain(|moment| moment.category != ForceCategory::Gravitational);

        // Store forces before clearing
        let original_forces = physics.forces.clone();
        let original_moments = physics.moments.clone();

        physics.clear_forces();

        // Process forces
        for force in original_forces {
            // Transform force to inertial frame
            let force_inertial = match force.frame {
                ReferenceFrame::Body => spatial.attitude * force.vector,
                ReferenceFrame::Inertial => force.vector,
                ReferenceFrame::Wind => spatial.attitude * force.vector,
            };

            // Add to net force
            physics.net_force += force_inertial;

            // Store transformed force
            physics.add_force(Force {
                vector: force_inertial,
                point: force.point,
                frame: ReferenceFrame::Inertial,
                category: force.category,
            });

            // Calculate moment if force has application point
            if let Some(point) = force.point {
                let point_inertial = spatial.attitude * point;
                let moment = point_inertial.cross(&force_inertial);
                physics.net_moment += moment;

                physics.add_moment(Moment {
                    vector: moment,
                    frame: ReferenceFrame::Inertial,
                    category: force.category.clone(),
                });
            }
        }

        // Process pure moments (transform from body to inertial frame)
        for moment in original_moments {
            let moment_inertial = match moment.frame {
                ReferenceFrame::Body => spatial.attitude * moment.vector,
                ReferenceFrame::Inertial => moment.vector,
                ReferenceFrame::Wind => spatial.attitude * moment.vector,
            };

            // Add to net moment
            physics.net_moment += moment_inertial;

            // Store transformed moment
            physics.add_moment(Moment {
                vector: moment_inertial,
                frame: ReferenceFrame::Inertial,
                category: moment.category,
            });
        }

        // Add gravitational force (already in inertial frame)
        let gravity_force = Force {
            vector: config.gravity * physics.mass,
            point: None,
            frame: ReferenceFrame::Inertial,
            category: ForceCategory::Gravitational,
        };
        physics.add_force(gravity_force.clone());
        physics.net_force += gravity_force.vector;

        // println!("Final net force: {:?}", physics.net_force);
        // println!("Final net moment: {:?}", physics.net_moment);
    }
}
