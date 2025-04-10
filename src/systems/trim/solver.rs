use bevy::prelude::*;

use crate::{
    components::{
        AirData, AircraftControlSurfaces, FullAircraftConfig, NeedsTrim, PropulsionState, 
        SpatialComponent, TrimSolver, TrimSolverConfig, TrimStage, TrimState,
    },
    resources::PhysicsConfig,
};

/// System that handles aircraft trim operations
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
    // Static counter to track trim system iterations 
    static mut DEBUG_ITERATIONS: usize = 0;

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
                needs_trim.condition,
                &physics_config,
            );

            // Initialize with current state
            let initial_guess = TrimState::to_trim_state(&spatial, &controls, &air_data);
            solver.initialize(initial_guess);

            needs_trim.solver = Some(solver);
            return; // Return after initialization to allow proper setup
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
                        
                        // Add debug info about the actual state at the end
                        println!("\nDEBUG: FINAL AIRCRAFT STATE VALUES:");
                        println!("  Optimizer Trim State: elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°",
                            result.state.longitudinal.elevator,
                            result.state.longitudinal.power_lever,
                            result.state.longitudinal.alpha.to_degrees(),
                            result.state.longitudinal.theta.to_degrees()
                        );
                        println!("  Actual Sim State: elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°, vel=[{:.1}, {:.1}, {:.1}]",
                            controls.elevator,
                            controls.power_lever,
                            air_data.alpha.to_degrees(),
                            spatial.attitude.euler_angles().1.to_degrees(),
                            spatial.velocity.x, spatial.velocity.y, spatial.velocity.z
                        );
                        
                        // Update trim stage
                        if needs_trim.stage == TrimStage::Longitudinal {
                            needs_trim.stage = TrimStage::Complete;
                            commands.entity(entity).remove::<NeedsTrim>();
                        }
                    }
                }
                Ok(false) => {
                    // Continue iterating
                    let current = solver.current_state();
                    current.apply_trim_state(&mut controls, &mut air_data, &mut spatial);
                    
                    // Add periodic debug output to monitor simulation state
                    unsafe {
                        DEBUG_ITERATIONS += 1;
                        if DEBUG_ITERATIONS % 50 == 0 {
                            println!("\nDEBUG: AIRCRAFT STATE VALUES AT ITERATION {}:", DEBUG_ITERATIONS);
                            println!("  Optimizer State: elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°",
                                current.longitudinal.elevator,
                                current.longitudinal.power_lever,
                                current.longitudinal.alpha.to_degrees(),
                                current.longitudinal.theta.to_degrees()
                            );
                            println!("  Actual Sim State: elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°, vel=[{:.1}, {:.1}, {:.1}]",
                                controls.elevator,
                                controls.power_lever,
                                air_data.alpha.to_degrees(),
                                spatial.attitude.euler_angles().1.to_degrees(),
                                spatial.velocity.x, spatial.velocity.y, spatial.velocity.z
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Trim solver error: {:?}", e);
                    commands.entity(entity).remove::<NeedsTrim>();
                }
            }
        }
    }
}
