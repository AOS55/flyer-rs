use bevy::prelude::*;
use flyer::{
    components::{
        AirData, AircraftControlSurfaces, FixedStartConfig, FullAircraftConfig,
        PhysicsComponent, SpatialComponent, StartConfig,
    },
    plugins::{EnvironmentPlugin, FullAircraftPlugin, PhysicsPlugin, TransformationPlugin},
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, 
        physics_integrator_system,
    },
};
use nalgebra::Vector3;
use std::{fs::OpenOptions, io::Write, path::PathBuf};

// Resource to track simulation state
#[derive(Resource)]
struct SimulationState {
    time: f32,
    log_file: PathBuf,
}

impl Default for SimulationState {
    fn default() -> Self {
        // Create log file
        let log_file = std::env::temp_dir().join("aircraft_rollout.csv");
        
        // Initialize log file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file) 
        {
            writeln!(
                file,
                "time,airspeed,altitude,theta,alpha,elevator,throttle,vertical_accel,horizontal_accel"
            ).unwrap();
        }
        
        println!("Log file created at: {:?}", log_file);
        
        Self {
            time: 0.0,
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
    
    // Add required plugins
    app.add_plugins((
        PhysicsPlugin::with_config(PhysicsConfig {
            timestep: physics_dt,
            ..default()
        }),
        TransformationPlugin::new(1.0),
        EnvironmentPlugin::new(),
    ));

    // Create and configure aircraft
    let mut aircraft_config = FullAircraftConfig::default();
    aircraft_config.start_config = StartConfig::Fixed(FixedStartConfig {
        position: Vector3::new(0.0, 0.0, -1000.0), // 1000m altitude
        speed: 70.0,                               // 70 m/s initial speed
        heading: 0.0,                              // North
    });

    // Add aircraft
    app.add_plugins(FullAircraftPlugin::new_single(aircraft_config));
    
    // Add simulation state tracker
    app.insert_resource(SimulationState::default());

    // Add physics and logging systems
    app.add_systems(
        FixedUpdate,
        (
            // Physics systems
            air_data_system,
            aero_force_system,
            force_calculator_system,
            physics_integrator_system,
            
            // Apply initial trim settings
            apply_trim_settings.run_if(on_startup()),
            
            // Logging system
            log_state,
        ).chain(),
    );

    println!("Starting simulation...");
    println!("Using pre-defined trim settings for straight and level flight at 70 m/s");
    
    // Run simulation for 10 seconds
    let max_time = 10.0;
    while app.world_mut().resource::<SimulationState>().time < max_time {
        app.update();
    }
    
    println!("Simulation complete!");
    println!("Data logged to: {:?}", app.world().resource::<SimulationState>().log_file);
}

// Apply trim settings to the aircraft
fn apply_trim_settings(
    mut query: Query<&mut AircraftControlSurfaces>,
) {
    if let Ok(mut controls) = query.get_single_mut() {
        // Use known good trim values for 70 m/s straight and level flight
        // These values come from previous trim calculations
        controls.elevator = -0.05;    // Slight up elevator
        controls.power_lever = 0.75;  // 75% throttle
        
        // Keep lateral controls neutral
        controls.aileron = 0.0;
        controls.rudder = 0.0;
        
        println!("Applied pre-calculated trim settings:");
        println!("  Elevator: {:.2}", controls.elevator);
        println!("  Throttle: {:.2}", controls.power_lever);
        println!("  Aileron: {:.2}", controls.aileron);
        println!("  Rudder: {:.2}", controls.rudder);
    }
}

// Log aircraft state for analysis
fn log_state(
    query: Query<(&SpatialComponent, &AircraftControlSurfaces, &AirData, &PhysicsComponent)>,
    mut state: ResMut<SimulationState>,
    time: Res<Time>,
) {
    // Update simulation time
    let dt = time.delta_secs();
    state.time += dt;
    
    if let Ok((spatial, controls, air_data, physics)) = query.get_single() {
        // Extract pitch angle
        let (_, pitch, _) = spatial.attitude.euler_angles();
        
        // Get accelerations directly from physics component
        let vertical_accel = physics.forces.iter()
            .filter(|force| force.frame.is_body())
            .map(|force| force.vector.z)
            .sum::<f64>() / physics.mass;
            
        let horizontal_accel = physics.forces.iter()
            .filter(|force| force.frame.is_body())
            .map(|force| force.vector.x)
            .sum::<f64>() / physics.mass;
        
        // Log data
        if let Ok(mut file) = OpenOptions::new().append(true).open(&state.log_file) {
            writeln!(
                file,
                "{:.3},{:.2},{:.2},{:.2},{:.2},{:.4},{:.4},{:.4},{:.4}",
                state.time,
                air_data.true_airspeed,
                -spatial.position.z,
                pitch.to_degrees(),
                air_data.alpha.to_degrees(),
                controls.elevator,
                controls.power_lever,
                vertical_accel,
                horizontal_accel
            ).unwrap();
        }
        
        // Print current state periodically
        if (state.time * 10.0).round() / 10.0 == state.time {  // Print every 0.1s
            println!(
                "Time: {:.1}s | Alt: {:.0}m | Speed: {:.1}m/s | Pitch: {:.1}° | Alpha: {:.1}°",
                state.time,
                -spatial.position.z,
                air_data.true_airspeed,
                pitch.to_degrees(),
                air_data.alpha.to_degrees()
            );
        }
    }
}
