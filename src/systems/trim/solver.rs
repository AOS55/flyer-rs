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
    for (entity, mut controls, mut spatial, propulsion, mut air_data, config, mut needs_trim) in
        query.iter_mut()
    {
        // Initialize solver if needed
        if needs_trim.solver.is_none() {
            let mut solver = TrimSolver::new(
                settings.clone(),
                spatial.clone(),
                propulsion.clone(),
                config.clone(),
                needs_trim.condition, // Changed order
                &physics_config,
            );

            // Initialize with current state
            let initial_guess = TrimState::to_trim_state(&spatial, &controls, &air_data);
            solver.initialize(initial_guess);

            needs_trim.solver = Some(solver);
            return;
        }

        // Run iteration
        if let Some(ref mut solver) = &mut needs_trim.solver {
            match solver.iterate() {
                Ok(true) => {
                    // Converged
                    let result = solver.get_best_solution();
                    if result.converged {
                        result
                            .state
                            .apply_trim_state(&mut controls, &mut air_data, &mut spatial);
                        info!(
                            "Aircraft {:?} trimmed successfully:\n\
                            Longitudinal: elevator={:.3}, throttle={:.3}, alpha={:.1}°, theta={:.1}°\n\
                            Cost: {:.6}, Iterations: {}",
                            config.name,
                            result.state.longitudinal.elevator,
                            result.state.longitudinal.power_lever,
                            result.state.longitudinal.alpha.to_degrees(),
                            result.state.longitudinal.theta.to_degrees(),
                            result.cost,
                            result.iterations
                        );
                        commands.entity(entity).remove::<NeedsTrim>();
                    }
                }
                Ok(false) => {
                    // Continue iterating
                    let current = solver.current_state;
                    current.apply_trim_state(&mut controls, &mut air_data, &mut spatial);
                }
                Err(e) => {
                    error!("Trim solver error: {:?}", e);
                    commands.entity(entity).remove::<NeedsTrim>();
                }
            }
        }
    }
}
