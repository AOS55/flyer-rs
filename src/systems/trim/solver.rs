use bevy::prelude::*;

use crate::{
    components::{
        FullAircraftConfig, FullAircraftState, NeedsTrim, TrimSolver, TrimSolverConfig,
        TrimStateConversion,
    },
    resources::PhysicsConfig,
};

pub fn trim_aircraft_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut FullAircraftState,
        &FullAircraftConfig,
        &mut NeedsTrim,
    )>,
    physics_config: Res<PhysicsConfig>,
    settings: Res<TrimSolverConfig>,
) {
    for (entity, mut state, config, mut needs_trim) in query.iter_mut() {
        // Initialize solver if this is the first time
        if needs_trim.solver.is_none() {
            let mut solver = TrimSolver::new(
                settings.clone(),
                config.clone(),
                needs_trim.condition,
                physics_config.clone(),
            );

            // Use current state as initial guess
            let initial_guess = state.to_trim_state();
            solver.initialize(initial_guess);
            needs_trim.solver = Some(solver);
        }

        // Run a few iterations each frame
        if let Some(ref mut solver) = &mut needs_trim.solver {
            // Do a few iterations per frame to avoid stalling
            for _ in 0..5 {
                if !solver.has_converged() && solver.iteration < settings.max_iterations {
                    solver.iterate();
                }
            }

            // Check if we're done
            if solver.has_converged() || solver.iteration >= settings.max_iterations {
                let result = solver.get_best_solution();
                if result.converged {
                    // Update aircraft state
                    state.apply_trim_state(&result.state);

                    info!(
                        "Aircraft {:?} trimmed successfully:\n\
                                Elevator: {:.3}, Aileron: {:.3}, Rudder: {:.3}, Throttle: {:.3}\n\
                                Alpha: {:.2}°, Beta: {:.2}°, Phi: {:.2}°, Theta: {:.2}°\n\
                                Cost: {:.6}, Iterations: {}",
                        config.name,
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
                } else {
                    warn!(
                        "Trim solver failed to converge for aircraft {:?} after {} iterations with cost {}",
                        config.name, result.iterations, result.cost
                    );
                }
                // Remove NeedsTrim component when done
                commands.entity(entity).remove::<NeedsTrim>();
            }
        }
    }
}
