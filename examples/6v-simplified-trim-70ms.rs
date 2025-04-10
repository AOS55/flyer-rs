use bevy::prelude::*;
use flyer::{
    components::{
        AirData, AircraftControlSurfaces, FullAircraftConfig, NeedsTrim, PropulsionState,
        SpatialComponent, TrimCondition, TrimRequest, TrimSolverConfig,
    },
    plugins::{EnvironmentPlugin, PhysicsPlugin, TransformationPlugin},
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, handle_trim_requests,
        physics_integrator_system, trim_aircraft_system,
    },
};
use nalgebra::{UnitQuaternion, Vector3};

// Define tracking resource for convergence monitoring
#[derive(Resource)]
struct TrimConvergenceTracker {
    last_cost: f64,
    iterations: usize,
    trim_complete: bool,
    cost_history: Vec<f64>,
    record_interval: usize,
}

impl Default for TrimConvergenceTracker {
    fn default() -> Self {
        Self {
            last_cost: f64::INFINITY,
            iterations: 0,
            trim_complete: false,
            cost_history: Vec::new(),
            record_interval: 1, // Record every iteration for better visibility
        }
    }
}

fn main() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    // Use a fixed timestep (100Hz)
    app.insert_resource(Time::<Fixed>::from_hz(100.0));
    println!("Using fixed timestep of 1/100 second");

    // Set up trim request event
    app.add_event::<TrimRequest>();

    // Configure physics
    let physics_config = PhysicsConfig::default();
    app.add_plugins((
        PhysicsPlugin::with_config(physics_config.clone()),
        TransformationPlugin::default(),
        EnvironmentPlugin::new(),
    ));

    // Add simplified trim solver config
    let trim_config = TrimSolverConfig {
        max_iterations: 50,  // Limit iterations for testing
        cost_tolerance: 1.0, // More reasonable tolerance given the force magnitudes
        debug_level: 0,      // Turn off debugging output to reduce console spam
        ..Default::default()
    };
    app.insert_resource(trim_config);

    // Add trim convergence tracker
    app.insert_resource(TrimConvergenceTracker::default());

    // Register systems
    app.add_systems(
        Update,
        (
            // Trim systems
            handle_trim_requests,
            trim_aircraft_system,
            // Physics systems
            air_data_system,
            aero_force_system,
            force_calculator_system,
            physics_integrator_system,
            // Trim monitoring
            request_trim,
            monitor_trim_convergence.after(trim_aircraft_system),
        ),
    );

    // Spawn aircraft entity
    app.add_systems(Startup, setup_aircraft);

    // Run the app
    println!("Starting simulation...");
    app.run();
}

// System to set up the aircraft
fn setup_aircraft(mut commands: Commands) {
    // Create an aircraft with initial state for 55 m/s
    let entity = commands
        .spawn((
            // Spatial component with initial velocity (55 m/s along X axis)
            SpatialComponent {
                position: Vector3::new(0.0, 1000.0, 0.0), // 1000m altitude
                velocity: Vector3::new(55.0, 0.0, 0.0),   // 55 m/s along X axis
                attitude: UnitQuaternion::identity(),     // Level attitude
                angular_velocity: Vector3::zeros(),
            },
            // Default control surfaces
            AircraftControlSurfaces {
                elevator: 0.0,
                aileron: 0.0,
                rudder: 0.0,
                power_lever: 0.5, // Start at 50% throttle
            },
            // Default propulsion state
            PropulsionState::default(),
            // Default air data
            AirData::default(),
            // Use Twin Otter configuration
            FullAircraftConfig::twin_otter(),
        ))
        .id();

    println!(
        "Twin Otter entity {:?} created at 1000m altitude with 55 m/s airspeed",
        entity
    );
}

// System to request trim for aircraft
fn request_trim(
    mut trim_requests: EventWriter<TrimRequest>,
    query: Query<Entity, With<SpatialComponent>>,
) {
    static mut ALREADY_REQUESTED: bool = false;
    let already_requested = unsafe { ALREADY_REQUESTED };

    if already_requested {
        return;
    }

    // Only request trim once
    unsafe {
        ALREADY_REQUESTED = true;
    }

    for entity in query.iter() {
        println!("Requesting trim for entity {:?}", entity);

        // Send straight and level trim request at 55 m/s for Twin Otter
        trim_requests.send(TrimRequest {
            entity,
            condition: TrimCondition::StraightAndLevel { airspeed: 55.0 },
        });

        println!("Trim request sent for straight and level flight at 55 m/s");
    }
}

// System to monitor trim convergence
fn monitor_trim_convergence(
    query: Query<(
        &SpatialComponent,
        &AircraftControlSurfaces,
        &AirData,
        Option<&NeedsTrim>,
    )>,
    mut tracker: ResMut<TrimConvergenceTracker>,
) {
    let should_print = tracker.iterations % 20 == 0; // Print less frequently
    let should_record = tracker.iterations % tracker.record_interval == 0; // Record at regular intervals

    for (spatial, controls, air_data, needs_trim) in query.iter() {
        if let Some(needs_trim) = needs_trim {
            if let Some(ref solver) = needs_trim.solver {
                // Get the best solution to access cost
                let result = solver.get_best_solution();
                let current_cost = result.cost;

                // Check for valid cost before proceeding
                if !current_cost.is_finite() {
                    // Only print warning if we're past initial setup
                    if tracker.iterations > 0 {
                        println!(
                            "Warning: Cost became non-finite at iteration {}",
                            tracker.iterations
                        );
                    }
                    return;
                }

                // Always record cost at specified intervals for tracking trend
                if should_record {
                    tracker.cost_history.push(current_cost);
                }

                // Increment iteration counter
                tracker.iterations += 1;

                // Print status if cost changed significantly or periodically
                if (tracker.last_cost - current_cost).abs() > 1e-2 || should_print {
                    // Extract relevant state parameters
                    let (_, pitch, _) = spatial.attitude.euler_angles();

                    println!(
                        "Iteration {}: Cost = {:.6}\n  Airspeed = {:.1} m/s, Alpha = {:.1}°, Beta = {:.1}°, Pitch = {:.1}°\n  Elevator = {:.3}, Aileron = {:.3}, Rudder = {:.3}, Throttle = {:.3}",
                        tracker.iterations,
                        current_cost,
                        spatial.velocity.norm(),
                        air_data.alpha.to_degrees(),
                        air_data.beta.to_degrees(),
                        pitch.to_degrees(),
                        controls.elevator,
                        controls.aileron,
                        controls.rudder,
                        controls.power_lever
                    );

                    tracker.last_cost = current_cost;
                }
            }
        } else if !tracker.trim_complete {
            // Trim has been completed (NeedsTrim component removed)
            tracker.trim_complete = true;

            // Extract relevant state parameters for final report
            let (roll, pitch, yaw) = spatial.attitude.euler_angles();

            println!("\n==== TRIM COMPLETED ====");
            println!(
                "Final state after {} iterations:\n  Airspeed = {:.1} m/s\n  Position: X={:.1}, Y={:.1}, Z={:.1}\n  Attitude: Roll={:.1}°, Pitch={:.1}°, Yaw={:.1}°\n  Alpha = {:.2}°, Beta = {:.2}°\n  Elevator = {:.3}, Aileron = {:.3}, Rudder = {:.3}, Throttle = {:.3}\n  Final Cost = {:.6}",
                tracker.iterations,
                spatial.velocity.norm(),
                spatial.position.x, spatial.position.y, spatial.position.z,
                roll.to_degrees(), pitch.to_degrees(), yaw.to_degrees(),
                air_data.alpha.to_degrees(), air_data.beta.to_degrees(),
                controls.elevator, controls.aileron, controls.rudder, controls.power_lever,
                tracker.last_cost
            );

            // Display cost progression summary
            println!("\nCost Progression Summary:");
            if tracker.cost_history.len() > 1 {
                let initial_cost = *tracker.cost_history.first().unwrap_or(&f64::INFINITY);
                let final_cost = *tracker.cost_history.last().unwrap_or(&tracker.last_cost);
                let total_reduction = initial_cost - final_cost;
                let percent_reduction = if initial_cost != 0.0 {
                    (total_reduction / initial_cost) * 100.0
                } else {
                    0.0
                };

                println!("  Initial Cost: {:.6}", initial_cost);
                println!("  Final Cost: {:.6}", final_cost);
                println!(
                    "  Total Reduction: {:.6} ({:.2}%)",
                    total_reduction, percent_reduction
                );

                // Check if cost consistently decreased
                let mut is_decreasing = true;
                for i in 1..tracker.cost_history.len() {
                    if tracker.cost_history[i] > tracker.cost_history[i - 1] {
                        is_decreasing = false;
                        break;
                    }
                }

                println!(
                    "  Cost Consistently Decreased: {}",
                    if is_decreasing { "Yes ✅" } else { "No ❌" }
                );
            } else {
                println!("  Insufficient data points to analyze cost progression");
            }

            // Evaluate the trim quality
            let airspeed_error = (spatial.velocity.norm() - 55.0).abs();
            let elevator_in_range = controls.elevator.abs() <= 1.0;
            let throttle_in_range = controls.power_lever >= 0.0 && controls.power_lever <= 1.0;

            println!("\nTrim Quality Assessment:");
            println!(
                "  Airspeed Error: {:.2} m/s (target: 55.0 m/s)",
                airspeed_error
            );
            println!(
                "  Elevator Position Valid: {} (value: {:.3})",
                elevator_in_range, controls.elevator
            );
            println!(
                "  Throttle Position Valid: {} (value: {:.3})",
                throttle_in_range, controls.power_lever
            );

            if airspeed_error < 1.0 && elevator_in_range && throttle_in_range {
                println!("\n✅ TRIM SUCCESSFUL: Aircraft trimmed correctly for straight and level flight at 55 m/s");
            } else {
                println!("\n❌ TRIM ISSUES: The trim solution may not be fully converged or valid");
            }

            println!("\nSimulation complete. Simplified trim solver demonstration finished.");
        }
    }
}
