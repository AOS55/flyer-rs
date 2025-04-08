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

// Flight phases for the demo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlightPhase {
    Trimming,        // Initial trimming phase
    StraightAndLevel, // Demonstrate stable flight with trimmed settings
    RollIn,          // Apply aileron for roll
    RollOut,         // Return to level flight
    Complete,        // Demo complete
}

// Resource to track simulation state and progress
#[derive(Resource)]
struct SimulationState {
    phase: FlightPhase,
    phase_time: f32,
    total_time: f32,
    trim_complete: bool,
    log_file: PathBuf,
}

impl Default for SimulationState {
    fn default() -> Self {
        // Create log file
        let log_file = std::env::temp_dir().join("aircraft_maneuver.csv");
        
        // Initialize log file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file) 
        {
            writeln!(
                file,
                "time,phase,airspeed,altitude,phi,theta,psi,alpha,beta,elevator,aileron,rudder,throttle,p,q,r"
            ).unwrap();
        }
        
        println!("Log file created at: {:?}", log_file);
        
        Self {
            phase: FlightPhase::Trimming,
            phase_time: 0.0,
            total_time: 0.0,
            trim_complete: false,
            log_file,
        }
    }
}

fn main() {
    let mut app = App::new();

    // Add minimal plugins without a window
    app.add_plugins(MinimalPlugins);

    // Set up physics with fixed timestep
    let physics_dt = 1.0 / 100.0; // 100Hz physics
    app.insert_resource(Time::<Fixed>::from_seconds(physics_dt));
    
    // Configure events
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

    // Configure trim solver
    app.insert_resource(TrimSolverConfig {
        max_iterations: 100,
        cost_tolerance: 1e-3,
        use_gradient_refinement: true,
        lateral_bounds: LateralBounds::default(),
        longitudinal_bounds: LongitudinalBounds {
            elevator_range: (-0.5, 0.5),
            throttle_range: (0.2, 0.9),
            alpha_range: (-0.2, 0.2),
            theta_range: (-0.3, 0.3),
        },
        debug_level: 1,
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
    app.insert_resource(SimulationState::default());

    // Add physics and control systems
    app.add_systems(
        FixedUpdate,
        (
            // Physics systems
            air_data_system,
            aero_force_system,
            force_calculator_system,
            physics_integrator_system,
            
            // Trim systems
            request_trim.run_if(|state: Res<SimulationState>| state.phase == FlightPhase::Trimming && !state.trim_complete),
            handle_trim_requests,
            trim_aircraft_system,
            
            // Control and monitoring systems
            update_flight_phase,
            apply_controls,
            log_state,
        ).chain(),
    );

    println!("Starting simulation...");
    println!("Phase 1: Trim aircraft");
    
    // Run simulation for just 5 seconds to focus on trim phase
    let max_time = 5.0;
    while app.world_mut().resource::<SimulationState>().total_time < max_time {
        println!("DEBUG: Running update at t={:.2}s, phase={:?}", 
            app.world().resource::<SimulationState>().total_time,
            app.world().resource::<SimulationState>().phase);
        
        app.update();
        
        // Check if trim changed after update
        if app.world().resource::<SimulationState>().phase != FlightPhase::Trimming {
            println!("DEBUG: Phase changed to {:?}", app.world().resource::<SimulationState>().phase);
        }
    }
    
    println!("Simulation complete!");
}

// Request trim for aircraft entities
fn request_trim(
    mut trim_requests: EventWriter<TrimRequest>,
    query: Query<Entity, With<AircraftControlSurfaces>>,
    mut state: ResMut<SimulationState>,
) {
    for entity in query.iter() {
        println!("Requesting straight and level trim at 50 m/s");
        
        // Send trim request
        trim_requests.send(TrimRequest {
            entity,
            condition: TrimCondition::StraightAndLevel { airspeed: 50.0 },
        });
        
        // Mark trim as in progress so we don't send multiple requests
        state.trim_complete = true;
    }
}

// Update flight phases based on timing and events
fn update_flight_phase(
    time: Res<Time>,
    query: Query<(&SpatialComponent, Option<&NeedsTrim>)>,
    mut state: ResMut<SimulationState>,
) {
    // Update timers
    let dt = time.delta_secs();
    state.total_time += dt;
    state.phase_time += dt;
    
    // Handle phase transitions
    match state.phase {
        FlightPhase::Trimming => {
            // Check if any aircraft still needs trim
            let mut all_trimmed = true;
            let mut aircraft_count = 0;
            
            for (_, needs_trim) in query.iter() {
                aircraft_count += 1;
                if needs_trim.is_some() {
                    all_trimmed = false;
                    // Debug print to see trim status
                    if state.total_time > 2.0 && state.total_time < 2.1 {
                        println!("DEBUG: Aircraft still being trimmed at t={:.1}s", state.total_time);
                    }
                    break;
                }
            }
            
            // Debug print to track aircraft entities
            if state.total_time > 0.9 && state.total_time < 1.1 {
                println!("DEBUG: Found {} aircraft entities", aircraft_count);
                println!("DEBUG: all_trimmed={}, trim_complete={}", all_trimmed, state.trim_complete);
            }
            
            // If all aircraft are trimmed and we've marked trim as complete, move to the next phase
            if all_trimmed && state.trim_complete && state.phase_time > 0.5 {
                println!("\nPhase 2: Demonstrating straight and level flight with trimmed settings");
                state.phase = FlightPhase::StraightAndLevel;
                state.phase_time = 0.0;
            }
        },
        FlightPhase::StraightAndLevel => {
            // After 5 seconds of stable flight, start the roll
            if state.phase_time > 5.0 {
                println!("\nPhase 3: Initiating roll maneuver (applying right aileron)");
                state.phase = FlightPhase::RollIn;
                state.phase_time = 0.0;
            }
        },
        FlightPhase::RollIn => {
            // After 3 seconds of roll, return to level
            if state.phase_time > 3.0 {
                println!("\nPhase 4: Roll-out maneuver (returning to level)");
                state.phase = FlightPhase::RollOut;
                state.phase_time = 0.0;
            }
        },
        FlightPhase::RollOut => {
            // After 5 seconds of roll-out, complete the demo
            if state.phase_time > 5.0 {
                println!("\nPhase 5: Demo complete");
                println!("Trimmed aircraft successfully controlled through roll maneuver and return to level flight");
                state.phase = FlightPhase::Complete;
            }
        },
        FlightPhase::Complete => {
            // Nothing to do
        }
    }
}

// Apply control inputs based on flight phase
fn apply_controls(
    mut query: Query<&mut AircraftControlSurfaces>,
    state: Res<SimulationState>,
) {
    if let Ok(mut controls) = query.get_single_mut() {
        match state.phase {
            FlightPhase::Trimming => {
                // During trimming, don't touch the controls - let the trim solver handle them
            },
            FlightPhase::StraightAndLevel => {
                // Controls are already set to trimmed values, don't change them
            },
            FlightPhase::RollIn => {
                // Apply right aileron for roll
                controls.aileron = 0.3; // 30% right aileron
                controls.rudder = 0.1;  // Small amount of rudder for coordination
            },
            FlightPhase::RollOut => {
                // Return to level flight
                controls.aileron = 0.0;
                controls.rudder = 0.0;
            },
            FlightPhase::Complete => {
                // Final state, no control changes
            }
        }
    }
}

// Log aircraft state for analysis
fn log_state(
    query: Query<(&SpatialComponent, &AircraftControlSurfaces, &AirData, &PhysicsComponent)>,
    state: Res<SimulationState>,
    time: Res<Time>,
) {
    if let Ok((spatial, controls, air_data, physics)) = query.get_single() {
        // Extract Euler angles
        let (roll, pitch, yaw) = spatial.attitude.euler_angles();
        
        // Angular velocity
        let p = spatial.angular_velocity.x;
        let q = spatial.angular_velocity.y;
        let r = spatial.angular_velocity.z;
        
        // Log data
        if let Ok(mut file) = OpenOptions::new().append(true).open(&state.log_file) {
            let phase_id = match state.phase {
                FlightPhase::Trimming => 1,
                FlightPhase::StraightAndLevel => 2,
                FlightPhase::RollIn => 3,
                FlightPhase::RollOut => 4,
                FlightPhase::Complete => 5,
            };
            
            writeln!(
                file,
                "{:.3},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
                state.total_time,
                phase_id,
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
                p,
                q,
                r
            ).unwrap();
        }
        
        // Print current state periodically
        static mut PRINT_COUNTER: u32 = 0;
        unsafe {
            PRINT_COUNTER += 1;
            if PRINT_COUNTER % 100 == 0 { // Print every 100 frames (about 1 second)
                println!(
                    "Time: {:.1}s | Phase: {:?} | Alt: {:.0}m | Speed: {:.1}m/s | Roll: {:.1}° | Pitch: {:.1}°",
                    state.total_time,
                    state.phase,
                    -spatial.position.z,
                    air_data.true_airspeed,
                    roll.to_degrees(),
                    pitch.to_degrees()
                );
            }
        }
    }
}