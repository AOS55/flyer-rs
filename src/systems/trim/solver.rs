use crate::{
    components::{
        AirData,
        AircraftControlSurfaces,
        FullAircraftConfig,
        LongitudinalTrimState, // Need this struct
        NeedsTrim,
        PhysicsComponent,
        PropulsionState,
        SpatialComponent,
        TrimCondition,
        TrimSolverConfig,
        TrimStage,
        TrimState,
    },
    resources::{EnvironmentModel, PhysicsConfig},
    systems::{
        calculate_engine_outputs, params_to_state_inputs, update_powerplant_state, TrimProblem,
    },
};
use argmin::{
    core::Executor,
    solver::{linesearch::MoreThuenteLineSearch, quasinewton::LBFGS},
};
use bevy::prelude::*;

/// System that performs aircraft trimming using argmin Executor and pure functions.
pub fn trim_aircraft_system(
    mut query: Query<(
        Entity,
        &mut AircraftControlSurfaces,
        &mut SpatialComponent,
        &mut AirData,
        &PhysicsComponent,
        &mut PropulsionState,
        &FullAircraftConfig,
        &mut NeedsTrim,
    )>,
    solver_config: Res<TrimSolverConfig>,
    physics_config: Res<PhysicsConfig>,
    environment: Res<EnvironmentModel>,
) {
    for (
        entity,
        mut controls,
        mut spatial,
        mut air_data,
        _physics,
        mut propulsion_state,
        aircraft_config,
        mut needs_trim,
    ) in query.iter_mut()
    {
        if needs_trim.stage != TrimStage::Longitudinal {
            continue;
        }

        // --- GET INITIAL DENSITY ---
        let initial_altitude = -spatial.position.z;
        let initial_density = environment.get_density_at_altitude(-initial_altitude);

        info!("Running Gradient-Based Trim for Entity: {:?}", entity);

        // --- 1. Setup Optimization Problem ---
        let problem = TrimProblem {
            aircraft_config: &aircraft_config,
            physics_config: &physics_config,
            solver_config: &solver_config,
            target_condition: needs_trim.condition,
            initial_spatial: spatial.clone(), // Clone state needed inside cost function
            initial_prop_state: propulsion_state.clone(),
            initial_density,
        };

        // --- 2. Initial Guess Vector ---
        // Order: [alpha, elevator, power_lever]
        let initial_state_guess = TrimState::to_trim_state(&spatial, &controls, &air_data);
        // Use helper to apply bounds immediately
        let (init_alpha, init_elevator, init_power_lever) = params_to_state_inputs(
            &[
                initial_state_guess.longitudinal.alpha,
                initial_state_guess.longitudinal.elevator,
                initial_state_guess.longitudinal.power_lever,
            ],
            &solver_config.longitudinal_bounds,
        );
        let init_param = vec![init_alpha, init_elevator, init_power_lever];
        info!(
            "Trim Initial Guess (Clamped): A={:.2}deg E={:.3} P={:.3}",
            init_alpha.to_degrees(),
            init_elevator,
            init_power_lever
        );

        // --- 3. Configure Solver (L-BFGS example) ---
        // Configure line search (optional, defaults are often okay)
        let linesearch = MoreThuenteLineSearch::new()
            .with_c(1e-4, 0.9) // Set Wolfe condition parameters
            .unwrap_or_else(|e| {
                warn!("Failed to configure linesearch: {:?}", e);
                MoreThuenteLineSearch::new()
            });
        // L-BFGS solver with memory size 7 (argmin default)
        let solver = LBFGS::new(linesearch, 7);

        // --- 4. Run Executor ---
        let max_iters = solver_config.max_iterations as u64;
        // Target cost is tolerance squared (since cost is sum of squares)
        // Use a slightly looser target for cost than individual residuals maybe
        let target_cost = solver_config.cost_tolerance.powi(2) * 10.0; // Example adjustment

        // Create executor with problem and solver and configure it
        // The correct pattern per Argmin documentation is:
        // 1. Create executor with new()
        // 2. Configure the state with configure()
        // 3. Run the optimization with run()
        let result = Executor::new(problem, solver)
            .configure(|state| {
                state
                    .param(init_param)
                    .max_iters(max_iters) // max_iters is already u64
                    .target_cost(target_cost)
            })
            .run();

        // --- 5. Process Result ---
        match result {
            Ok(opt_result) => {
                let final_state = opt_result.state();
                info!(
                    "Trim Optimization Finished: Terminated Reason: {:?}",
                    final_state.termination_status.to_string() // Handle Option
                );
                info!(
                    " Best Cost: {:.6e} (Target: {:.3e}), Iterations: {}",
                    final_state.best_cost, target_cost, final_state.iter
                );

                // Use the best parameters found
                if let Some(best_param) = final_state.best_param.as_ref() {
                    // Convert best parameters back to TrimState, clamping final values
                    let (final_alpha, final_elevator, final_power_lever) =
                        params_to_state_inputs(best_param, &solver_config.longitudinal_bounds);

                    let (target_gamma, _) = match needs_trim.condition {
                        TrimCondition::StraightAndLevel { .. } => (0.0, 0.0),
                        TrimCondition::SteadyClimb { gamma, .. } => (gamma, 0.0),
                        _ => (0.0, 0.0),
                    };
                    // Calculate final theta based on final alpha and gamma
                    let final_theta = (final_alpha + target_gamma).clamp(
                        solver_config.longitudinal_bounds.theta_range.0,
                        solver_config.longitudinal_bounds.theta_range.1,
                    );

                    // Create the final TrimState struct
                    let final_trim_state = TrimState {
                        longitudinal: LongitudinalTrimState {
                            alpha: final_alpha,
                            theta: final_theta,
                            elevator: final_elevator,
                            power_lever: final_power_lever,
                        },
                        lateral: Default::default(), // Zero lateral state
                    };

                    info!(
                        "Applying Final Trim State: A={:.2}deg T={:.2}deg E={:.3} P={:.3}",
                        final_alpha.to_degrees(),
                        final_theta.to_degrees(),
                        final_elevator,
                        final_power_lever
                    );

                    // Apply the final state to the actual aircraft components
                    final_trim_state.apply_trim_state(&mut controls, &mut air_data, &mut spatial);

                    // Update propulsion state based on final power level
                    propulsion_state.set_power_lever(final_power_lever);
                    for (engine_config, engine_state) in aircraft_config
                        .propulsion
                        .engines
                        .iter()
                        .zip(propulsion_state.engine_states.iter_mut())
                    {
                        engine_state.power_lever = final_power_lever;
                        // Set steady state fraction for the start after trim
                        update_powerplant_state(engine_state, engine_config, 0.0); // Update running status
                        engine_state.thrust_fraction = if engine_state.running {
                            final_power_lever
                        } else {
                            0.0
                        };
                        // Update fuel flow based on final state
                        let final_outputs = calculate_engine_outputs(
                            engine_config,
                            engine_state,
                            air_data.density,
                            air_data.true_airspeed,
                        );
                        engine_state.fuel_flow = final_outputs.fuel_flow;
                    }
                } else {
                    warn!(
                        "Trim optimization finished but no best parameters found for {:?}.",
                        entity
                    );
                }
            }
            Err(e) => {
                error!("Trim optimization failed for {:?}: {}", entity, e);
            }
        }

        // --- 6. Cleanup ---
        needs_trim.stage = TrimStage::Complete; // Mark as done
        info!("Trim calculation complete for {:?}.", entity);
        // commands.entity(entity).remove::<NeedsTrim>(); // Optionally remove component
    }
}
