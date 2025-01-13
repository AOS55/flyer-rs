use crate::components::{Force, ForceCategory, FullAircraftState, Moment, ReferenceFrame};
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
    mut query: Query<&mut FullAircraftState>,
    config: Res<PhysicsConfig>,
) {
    println!("Running Force Calculator System!");
    for mut state in query.iter_mut() {
        println!("physics: {:?}, spatial, {:?}", state, config);

        // Store forces before clearing
        let original_forces = state.physics.forces.clone();

        state.physics.clear_forces();

        // Process forces
        for force in original_forces {
            // Transform force to inertial frame
            let force_inertial = match force.frame {
                ReferenceFrame::Body => state.spatial.attitude * force.vector,
                ReferenceFrame::Inertial => force.vector,
                ReferenceFrame::Wind => state.spatial.attitude * force.vector,
            };

            // Add to net force
            state.physics.net_force += force_inertial;

            // Store transformed force
            state.physics.add_force(Force {
                vector: force_inertial,
                point: force.point,
                frame: ReferenceFrame::Inertial,
                category: force.category,
            });

            // Calculate moment if force has application point
            if let Some(point) = force.point {
                let point_inertial = state.spatial.attitude * point;
                let moment = point_inertial.cross(&force_inertial);
                state.physics.net_moment += moment;

                state.physics.add_moment(Moment {
                    vector: moment,
                    frame: ReferenceFrame::Inertial,
                    category: force.category.clone(),
                });
            }
        }

        // Add gravitational force (already in inertial frame)
        let gravity_force = Force {
            vector: config.gravity * state.physics.mass,
            point: None,
            frame: ReferenceFrame::Inertial,
            category: ForceCategory::Gravitational,
        };
        state.physics.add_force(gravity_force.clone());
        state.physics.net_force += gravity_force.vector;

        println!("Final net force: {:?}", state.physics.net_force);
        println!("Final net moment: {:?}", state.physics.net_moment);
    }
}
