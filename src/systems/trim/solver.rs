use bevy::prelude::*;

use crate::{
    components::{
        AirData, AircraftControlSurfaces, FullAircraftConfig, NeedsTrim, PropulsionState,
        SpatialComponent, TrimSolver, TrimSolverConfig, TrimState,
    },
    resources::PhysicsConfig,
};

pub fn trim_aircraft_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AircraftControlSurfaces,
        &mut SpatialComponent,
        &PropulsionState,
        &mut AirData,
        &FullAircraftConfig,
        &mut NeedsTrim,
    )>,
    physics_config: Res<PhysicsConfig>,
    settings: Res<TrimSolverConfig>,
) {
    for (
        entity,
        mut control_surfaces,
        mut spatial,
        propulsion,
        mut air_data,
        aircraft_config,
        mut needs_trim,
    ) in query.iter_mut()
    {
        // Initialize solver if this is the first time
        if needs_trim.solver.is_none() {
            let mut solver = TrimSolver::new(
                settings.clone(),
                spatial.clone(),
                propulsion.clone(),
                aircraft_config.clone(),
                needs_trim.condition,
                &physics_config.clone(),
            );

            // Use current state as initial guess
            let initial_guess = TrimState::to_trim_state(&spatial, &control_surfaces, &air_data);
            solver.initialize(initial_guess);
            needs_trim.solver = Some(solver);
            return;
        }

        // Run a few iterations each frame
        if let Some(ref mut solver) = &mut needs_trim.solver {
            for _ in 0..100 {
                if !solver.has_converged() && solver.iteration < settings.max_iterations {
                    solver.iterate();
                }
            }

            // Check if we're done
            if solver.has_converged() || solver.iteration >= settings.max_iterations {
                let result = solver.get_best_solution();
                if result.converged {
                    // Update aircraft state
                    result.state.apply_trim_state(
                        &mut control_surfaces,
                        &mut air_data, // TODO: Test if this should be being mutated
                        &mut spatial,  // TODO: Test if this should be being mutated
                    );

                    info!(
                        "Aircraft {:?} trimmed successfully:\n\
                                Elevator: {:.3}, Aileron: {:.3}, Rudder: {:.3}, Throttle: {:.3}\n\
                                Alpha: {:.2}°, Beta: {:.2}°, Phi: {:.2}°, Theta: {:.2}°\n\
                                Cost: {:.6}, Iterations: {}",
                        aircraft_config.name,
                        result.state.elevator,
                        result.state.aileron,
                        result.state.rudder,
                        result.state.power_lever,
                        result.state.alpha.to_degrees(),
                        result.state.beta.to_degrees(),
                        result.state.phi.to_degrees(),
                        result.state.theta.to_degrees(),
                        result.cost,
                        result.iterations
                    );

                    // Only remove component if successfully converged
                    commands.entity(entity).remove::<NeedsTrim>();
                } else {
                    warn!(
                        "Trim solver failed to converge for aircraft {:?} after {} iterations with cost {}",
                        aircraft_config.name, result.iterations, result.cost
                    );
                    // // Reset solver to try again with current best state
                    // let mut new_solver = TrimSolver::new(
                    //     settings.clone(),
                    //     spatial.clone(),
                    //     propulsion.clone(),
                    //     aircraft_config.clone(),
                    //     needs_trim.condition,
                    //     &physics_config.clone(),
                    // );
                    // new_solver.initialize(result.state);
                    // needs_trim.solver = Some(new_solver);
                }
            }
        }
    }
}
