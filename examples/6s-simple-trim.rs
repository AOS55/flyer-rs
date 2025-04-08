use bevy::prelude::*;
use flyer::{
    components::{
        AirData, AircraftControlSurfaces, FixedStartConfig, FullAircraftConfig, LateralBounds,
        LongitudinalBounds, NeedsTrim, PhysicsComponent, SpatialComponent, StartConfig,
        TrimCondition, TrimRequest, TrimSolverConfig,
    },
    plugins::{EnvironmentPlugin, FullAircraftPlugin, PhysicsPlugin, TransformationPlugin},
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, handle_trim_requests,
        physics_integrator_system, trim_aircraft_system,
    },
};
use nalgebra::Vector3;
use std::{fs::OpenOptions, io::Write, path::PathBuf};

// Simple trim verification example
// Demonstrates proper trim procedure with verbose debugging

#[derive(Resource)]
struct SimulationLogger {
    log_file: PathBuf,
    trim_requested: bool,
    trim_completed: bool,
    max_runtime: f32,
    verification_time: f32,
}

impl Default for SimulationLogger {
    fn default() -> Self {
        // Create log file
        let log_file = std::env::temp_dir().join("trim_verification.csv");
        
        // Initialize log file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file) 
        {
            writeln!(
                file,
                "time,airspeed,altitude,phi,theta,psi,alpha,beta,elevator,aileron,rudder,throttle,fx,fy,fz,mx,my,mz"
            ).unwrap();
        }
        
        println!("Log file created at: {:?}", log_file);
        
        Self {
            log_file,
            trim_requested: false,
            trim_completed: false,
            max_runtime: 20.0, // Run for 20 seconds to give trim more time
            verification_time: 5.0, // Spend 5 seconds in verification after trim
        }
    }
}

// Debug resource for tracking the trim process details
#[derive(Resource)]
struct TrimDebugInfo {
    debug_file: PathBuf,
    trimmed_states: Vec<(f32, TrimStateInfo)>, // (time, state_info)
    starting_values: Option<TrimStateInfo>,
    final_values: Option<TrimStateInfo>,
    trim_completed: bool,
    iterations_recorded: i32,
}

// State snapshot for debugging
#[derive(Clone, Debug)]
struct TrimStateInfo {
    elevator: f32,
    throttle: f32,
    alpha: f32,
    theta: f32,
    forces: Vector3<f32>,
    moments: Vector3<f32>,
    iteration: i32,
    cost: f32,
    residuals: (f32, f32, f32, f32), // vertical, horizontal, pitch, gamma
}

impl Default for TrimDebugInfo {
    fn default() -> Self {
        // Create debug file
        let debug_file = std::env::temp_dir().join("trim_debug_detail.csv");
        
        // Initialize debug file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&debug_file) 
        {
            writeln!(
                file,
                "time,iteration,elevator,throttle,alpha_deg,theta_deg,fx,fz,my,gamma_error,cost,vert_residual,horiz_residual,pitch_residual,gamma_residual"
            ).unwrap();
        }
        
        println!("Trim debug file created at: {:?}", debug_file);
        
        Self {
            debug_file,
            trimmed_states: Vec::new(),
            starting_values: None,
            final_values: None,
            trim_completed: false,
            iterations_recorded: 0,
        }
    }
}

fn main() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    // Set up physics with fixed timestep
    let physics_dt = 1.0 / 100.0; // 100Hz physics
    app.insert_resource(Time::<Fixed>::from_seconds(physics_dt));
    
    // Configure trim event
    app.add_event::<TrimRequest>();
    
    // Add required plugins
    app.add_plugins((
        PhysicsPlugin::with_config(PhysicsConfig {
            timestep: physics_dt,
            ..default()
        }),
        TransformationPlugin::new(1.0),
        EnvironmentPlugin::new(),
    ));

    // Configure trim solver with optimal settings
    app.insert_resource(TrimSolverConfig {
        max_iterations: 500,             // Increased from 300 to 500 for more optimization time
        cost_tolerance: 1e-5,            // Stricter tolerance (100x better)
        use_gradient_refinement: true,
        lateral_bounds: LateralBounds::default(),
        longitudinal_bounds: LongitudinalBounds {
            elevator_range: (-0.5, 0.5),
            throttle_range: (0.3, 0.9),  // Better minimum throttle
            alpha_range: (0.01, 0.2),    // Force positive alpha for level flight
            theta_range: (0.01, 0.3),    // Match theta range to alpha for level flight
        },
        debug_level: 2, // Full debug output
    });
    
    // Increase the max runtime to give more time for the trim to complete
    app.insert_resource(SimulationLogger {
        log_file: std::env::temp_dir().join("trim_verification.csv"),
        trim_requested: false,
        trim_completed: false,
        max_runtime: 20.0,   // Increase from 10 to 20 seconds
        verification_time: 5.0,
    });

    // Create and configure aircraft
    let mut aircraft_config = FullAircraftConfig::default();
    aircraft_config.start_config = StartConfig::Fixed(FixedStartConfig {
        position: Vector3::new(0.0, 0.0, -1000.0), // 1000m altitude
        speed: 50.0,                              // 50 m/s initial speed
        heading: 0.0,                             // North
    });

    // Add aircraft
    app.add_plugins(FullAircraftPlugin::new_single(aircraft_config));
    
    // Add simulation state tracker
    app.insert_resource(SimulationLogger::default());
    
    // Debug resource for tracking trim process
    app.insert_resource(TrimDebugInfo::default());

    // Add systems
    app.add_systems(
        FixedUpdate,
        (
            // Physics systems
            air_data_system,
            aero_force_system,
            force_calculator_system,
            physics_integrator_system,
            
            // Trim systems
            request_trim.run_if(|logger: Res<SimulationLogger>| !logger.trim_requested),
            handle_trim_requests,
            trim_aircraft_system,
            
            // Monitoring systems
            monitor_trim_status,
            log_aircraft_state,
            capture_trim_debug_info,
        ).chain(),
    );

    println!("\n===== SIMPLE TRIM DEMONSTRATION =====");
    println!("This example will:");
    println!("1. Configure an aircraft at 1000m altitude");
    println!("2. Request a trim for straight and level flight at 50 m/s");
    println!("3. Wait for trim to complete with full debugging");
    println!("4. Verify stability by maintaining trimmed state");
    println!("5. Log all states to a CSV file for analysis");
    
    let start_time = std::time::Instant::now();
    let mut total_time = 0.0;
    
    // Run simulation until timeout or completion
    while total_time < app.world().resource::<SimulationLogger>().max_runtime {
        app.update();
        total_time = start_time.elapsed().as_secs_f32();
    }
    
    println!("\n===== SIMULATION COMPLETE =====");
    println!("Total runtime: {:.1} seconds", total_time);
    println!("Results logged to: {:?}", app.world().resource::<SimulationLogger>().log_file);
}

// System to request trim once
fn request_trim(
    mut trim_requests: EventWriter<TrimRequest>,
    query: Query<Entity, With<AircraftControlSurfaces>>,
    mut logger: ResMut<SimulationLogger>,
) {
    println!("\n----- TRIM REQUEST INITIATED -----");
    
    for entity in query.iter() {
        println!("Requesting straight and level trim at 50 m/s for entity {:?}", entity);
        
        // Send trim request
        trim_requests.send(TrimRequest {
            entity,
            condition: TrimCondition::StraightAndLevel { airspeed: 50.0 },
        });
    }
    
    logger.trim_requested = true;
}

// Monitor trim status - both during and after trim
fn monitor_trim_status(
    query: Query<(Entity, &SpatialComponent, &AircraftControlSurfaces, &AirData, Option<&NeedsTrim>, Option<&PhysicsComponent>)>,
    mut logger: ResMut<SimulationLogger>,
) {
    for (entity, spatial, controls, air_data, needs_trim, physics) in query.iter() {
        // Check trim status
        if let Some(needs_trim) = needs_trim {
            if let Some(ref solver) = needs_trim.solver {
                // Print solver status every 1 second (100 frames at 100Hz)
                static mut TRIM_PRINT_COUNTER: u32 = 0;
                let should_print = unsafe {
                    TRIM_PRINT_COUNTER += 1;
                    TRIM_PRINT_COUNTER % 100 == 0
                };
                
                if should_print {
                    let (roll, pitch, yaw) = spatial.attitude.euler_angles();
                    
                    println!("\n----- TRIM IN PROGRESS -----");
                    println!("Entity: {:?}, Iterations: {}, Cost: {:.6}", 
                        entity, solver.iteration, solver.best_cost);
                    
                    println!("Current state:");
                    println!("  Speed: {:.1} m/s, Alpha: {:.1}°, Pitch: {:.1}°", 
                        spatial.velocity.norm(), 
                        air_data.alpha.to_degrees(), 
                        pitch.to_degrees());
                        
                    println!("  Controls: Elevator={:.3}, Throttle={:.3}", 
                        controls.elevator, controls.power_lever);
                }
            }
        } else if !logger.trim_completed {
            // Trim just completed
            let (roll, pitch, yaw) = spatial.attitude.euler_angles();
            
            println!("\n----- TRIM COMPLETED -----");
            println!("Entity: {:?}", entity);
            println!("Final state:");
            println!("  Speed: {:.2} m/s (target: 50.0 m/s)", spatial.velocity.norm());
            println!("  Altitude: {:.2} m", -spatial.position.z);
            println!("  Alpha: {:.2}°, Beta: {:.2}°", 
                air_data.alpha.to_degrees(), air_data.beta.to_degrees());
            println!("  Pitch: {:.2}°, Roll: {:.2}°, Yaw: {:.2}°", 
                pitch.to_degrees(), roll.to_degrees(), yaw.to_degrees());
            println!("Controls:");
            println!("  Elevator: {:.4}", controls.elevator);
            println!("  Aileron: {:.4}", controls.aileron);
            println!("  Rudder: {:.4}", controls.rudder);
            println!("  Throttle: {:.4}", controls.power_lever);
            
            println!("\n----- VERIFICATION PHASE STARTED -----");
            println!("Monitoring stability for {:.1} seconds with trimmed controls...", 
                logger.verification_time);
            
            logger.trim_completed = true;
        } else {
            // Verification phase - print status every 1 second
            static mut VERIFY_PRINT_COUNTER: u32 = 0;
            let should_print = unsafe {
                VERIFY_PRINT_COUNTER += 1;
                VERIFY_PRINT_COUNTER % 100 == 0
            };
            
            if should_print {
                let (roll, pitch, yaw) = spatial.attitude.euler_angles();
                
                println!("\n----- VERIFICATION STATUS -----");
                println!("Speed: {:.1} m/s, Altitude: {:.1} m", 
                    spatial.velocity.norm(), -spatial.position.z);
                println!("Alpha: {:.1}°, Pitch: {:.1}°, Roll: {:.1}°", 
                    air_data.alpha.to_degrees(), pitch.to_degrees(), roll.to_degrees());
                
                // Print force data if available
                if let Some(physics) = physics {
                    println!("Force balance:");
                    println!("  Forces: X={:.2}, Y={:.2}, Z={:.2} N",
                        physics.net_force.x, physics.net_force.y, physics.net_force.z);
                    println!("  Moments: X={:.2}, Y={:.2}, Z={:.2} Nm",
                        physics.net_moment.x, physics.net_moment.y, physics.net_moment.z);
                }
            }
        }
    }
}

// Log aircraft state to CSV
fn log_aircraft_state(
    query: Query<(&SpatialComponent, &AircraftControlSurfaces, &AirData, &PhysicsComponent)>,
    time: Res<Time>,
    logger: Res<SimulationLogger>,
) {
    if let Ok((spatial, controls, air_data, physics)) = query.get_single() {
        // Extract Euler angles
        let (roll, pitch, yaw) = spatial.attitude.euler_angles();
        
        // Log state to file
        if let Ok(mut file) = OpenOptions::new().append(true).open(&logger.log_file) {
            static mut LOG_COUNTER: u32 = 0;
            unsafe { LOG_COUNTER += 1; }
            
            // Log every 10 frames (10Hz) to avoid excessive data
            if unsafe { LOG_COUNTER % 10 == 0 } {
                // Use elapsed seconds from fixed time
                let elapsed = time.elapsed().as_secs_f32();
                
                writeln!(
                    file,
                    "{:.3},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.4},{:.4},{:.4},{:.4},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
                    elapsed,
                    air_data.true_airspeed,
                    -spatial.position.z,
                    roll.to_degrees(),
                    pitch.to_degrees(),
                    yaw.to_degrees(),
                    air_data.alpha.to_degrees(),
                    air_data.beta.to_degrees(),
                    controls.elevator,
                    controls.aileron,
                    controls.rudder,
                    controls.power_lever,
                    physics.net_force.x,
                    physics.net_force.y,
                    physics.net_force.z,
                    physics.net_moment.x,
                    physics.net_moment.y,
                    physics.net_moment.z
                ).unwrap();
            }
        }
    }
}

// Capture detailed trim optimization process information
fn capture_trim_debug_info(
    query: Query<(&SpatialComponent, &AircraftControlSurfaces, &AirData, &PhysicsComponent, Option<&NeedsTrim>)>,
    time: Res<Time>,
    mut debug_info: ResMut<TrimDebugInfo>,
) {
    if let Ok((spatial, controls, air_data, physics, needs_trim)) = query.get_single() {
        // Current time
        let elapsed = time.elapsed().as_secs_f32();
        
        // If trim is in progress, capture solver state
        if let Some(needs_trim) = needs_trim {
            if let Some(ref solver) = needs_trim.solver {
                // Calculate flight path angle from velocity
                let velocity = &spatial.velocity;
                let gamma = (-velocity.z).atan2(velocity.x);
                
                // Only record every other iteration to avoid too much data
                static mut RECORD_COUNTER: u32 = 0;
                let should_record = unsafe {
                    RECORD_COUNTER += 1;
                    RECORD_COUNTER % 2 == 0 // Record every other iteration
                };
                
                if should_record {
                    // Create state info
                    let state_info = TrimStateInfo {
                        elevator: controls.elevator as f32,
                        throttle: controls.power_lever as f32,
                        alpha: air_data.alpha as f32,
                        theta: spatial.attitude.euler_angles().1 as f32,
                        forces: physics.net_force.cast::<f32>(),
                        moments: physics.net_moment.cast::<f32>(),
                        iteration: solver.iteration as i32,
                        cost: solver.best_cost as f32,
                        residuals: (
                            physics.net_force.z as f32, 
                            physics.net_force.x as f32, 
                            physics.net_moment.y as f32,
                            gamma as f32
                        ),
                    };
                    
                    // Store first iteration state
                    if debug_info.starting_values.is_none() {
                        debug_info.starting_values = Some(state_info.clone());
                        println!("\n[DEBUG] Captured initial trim state at iteration {}", solver.iteration);
                    }
                    
                    // Store state with timestamp
                    debug_info.trimmed_states.push((elapsed, state_info));
                    debug_info.iterations_recorded += 1;
                    
                    // Log to file
                    if let Ok(mut file) = OpenOptions::new().append(true).open(&debug_info.debug_file) {
                        let state = &debug_info.trimmed_states.last().unwrap().1;
                        
                        writeln!(
                            file,
                            "{:.3},{},{:.4},{:.4},{:.2},{:.2},{:.2},{:.2},{:.2},{:.4},{:.6},{:.4},{:.4},{:.4},{:.4}",
                            elapsed,
                            state.iteration,
                            state.elevator,
                            state.throttle,
                            state.alpha.to_degrees(),
                            state.theta.to_degrees(),
                            state.forces.x,
                            state.forces.z,
                            state.moments.y,
                            state.residuals.3,
                            state.cost,
                            state.residuals.0,
                            state.residuals.1,
                            state.residuals.2,
                            state.residuals.3
                        ).unwrap();
                    }
                }
            }
        } 
        // If trim just completed and we haven't stored the final state
        else if !debug_info.trim_completed && debug_info.starting_values.is_some() {
            // Calculate flight path angle
            let gamma = (-spatial.velocity.z).atan2(spatial.velocity.x);
            
            // Store final state
            let final_state = TrimStateInfo {
                elevator: controls.elevator as f32,
                throttle: controls.power_lever as f32,
                alpha: air_data.alpha as f32,
                theta: spatial.attitude.euler_angles().1 as f32,
                forces: physics.net_force.cast::<f32>(),
                moments: physics.net_moment.cast::<f32>(),
                iteration: debug_info.iterations_recorded,
                cost: 0.0, // Can't know final cost since solver is gone
                residuals: (
                    physics.net_force.z as f32, 
                    physics.net_force.x as f32, 
                    physics.net_moment.y as f32,
                    gamma as f32
                ),
            };
            
            // Compare initial and final states
            if let Some(ref initial) = debug_info.starting_values {
                println!("\n===== TRIM DEBUGGING REPORT =====");
                println!("Iterations recorded: {}", debug_info.iterations_recorded);
                println!("\nInitial state:");
                println!("  Elevator: {:.4}, Throttle: {:.4}", initial.elevator, initial.throttle);
                println!("  Alpha: {:.2}°, Theta: {:.2}°", 
                    initial.alpha.to_degrees(), initial.theta.to_degrees());
                println!("  Forces: X={:.2}, Z={:.2}, My={:.2}", 
                    initial.forces.x, initial.forces.z, initial.moments.y);
                    
                println!("\nFinal state:");
                println!("  Elevator: {:.4}, Throttle: {:.4}", final_state.elevator, final_state.throttle);
                println!("  Alpha: {:.2}°, Theta: {:.2}°", 
                    final_state.alpha.to_degrees(), final_state.theta.to_degrees());
                println!("  Forces: X={:.2}, Z={:.2}, My={:.2}", 
                    final_state.forces.x, final_state.forces.z, final_state.moments.y);
                    
                // Check if values actually changed (optimization happened)
                let elevator_change = (final_state.elevator - initial.elevator).abs();
                let throttle_change = (final_state.throttle - initial.throttle).abs();
                let alpha_change = (final_state.alpha - initial.alpha).abs();
                let theta_change = (final_state.theta - initial.theta).abs();
                
                println!("\nParameter changes:");
                println!("  Elevator: {:.4} ({:.1}%)", 
                    elevator_change, 100.0 * elevator_change / initial.elevator.abs().max(0.001));
                println!("  Throttle: {:.4} ({:.1}%)", 
                    throttle_change, 100.0 * throttle_change / initial.throttle.abs().max(0.001));
                println!("  Alpha: {:.4}° ({:.1}%)", 
                    alpha_change.to_degrees(), 
                    100.0 * alpha_change / initial.alpha.abs().max(0.001));
                println!("  Theta: {:.4}° ({:.1}%)", 
                    theta_change.to_degrees(),
                    100.0 * theta_change / initial.theta.abs().max(0.001));
                    
                println!("\nForce balance improvement:");
                println!("  X-force: {:.2} -> {:.2} ({:.1}% change)", 
                    initial.forces.x, final_state.forces.x,
                    100.0 * (final_state.forces.x - initial.forces.x).abs() / initial.forces.x.abs().max(0.001));
                println!("  Z-force: {:.2} -> {:.2} ({:.1}% change)", 
                    initial.forces.z, final_state.forces.z,
                    100.0 * (final_state.forces.z - initial.forces.z).abs() / initial.forces.z.abs().max(0.001));
                println!("  Pitch moment: {:.2} -> {:.2} ({:.1}% change)", 
                    initial.moments.y, final_state.moments.y,
                    100.0 * (final_state.moments.y - initial.moments.y).abs() / initial.moments.y.abs().max(0.001));
                    
                println!("\nStability assessment:");
                println!("  Force balance: {}", 
                    if final_state.forces.norm() < 10.0 { "Good" } 
                    else if final_state.forces.norm() < 50.0 { "Moderate" } 
                    else { "Poor" });
                println!("  Moment balance: {}", 
                    if final_state.moments.norm() < 5.0 { "Good" } 
                    else if final_state.moments.norm() < 20.0 { "Moderate" } 
                    else { "Poor" });
                println!("  Param convergence: {}", 
                    if elevator_change + throttle_change > 0.05 { "Good" } 
                    else { "Poor - params barely changed!" });
            }
            
            debug_info.final_values = Some(final_state);
            debug_info.trim_completed = true;
            
            println!("\nDebug data written to: {:?}", debug_info.debug_file);
        }
    }
}