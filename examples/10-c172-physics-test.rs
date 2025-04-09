use bevy::prelude::*;
use flyer::{
    components::{
        AirData, AircraftControlSurfaces, FullAircraftConfig, PhysicsComponent, 
        PropulsionState, SpatialComponent, Force, ForceCategory, ReferenceFrame,
    },
    plugins::{EnvironmentPlugin, PhysicsPlugin, TransformationPlugin},
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, physics_integrator_system,
    },
};
use nalgebra::{UnitQuaternion, Vector3};
use std::fs::File;
use std::io::Write;

// Define resource for tracking physics over time
#[derive(Resource)]
struct PhysicsTracker {
    time_steps: usize,
    max_steps: usize,
    data_points: Vec<PhysicsDataPoint>,
    log_file: Option<File>,
}

// Structure to hold physics data at each time step
#[derive(Debug, Clone)]
struct PhysicsDataPoint {
    time: f32,
    position: Vector3<f64>,
    velocity: Vector3<f64>,
    attitude: (f64, f64, f64), // (roll, pitch, yaw) in degrees
    alpha: f64,  // degrees
    beta: f64,   // degrees
    forces: Vector3<f64>,
    moments: Vector3<f64>,
    total_aero_force: Vector3<f64>,
    gravity_force: Vector3<f64>,
    thrust_force: Vector3<f64>,
    controls: (f64, f64, f64, f64), // (elevator, aileron, rudder, power_lever)
}

impl Default for PhysicsTracker {
    fn default() -> Self {
        // Create and open log file
        let log_file = File::create("c172_physics_log.csv").ok();
        
        // Write header if file was created successfully
        if let Some(ref mut file) = log_file {
            let header = "time,pos_x,pos_y,pos_z,vel_x,vel_y,vel_z,roll,pitch,yaw,alpha,beta,force_x,force_y,force_z,moment_x,moment_y,moment_z,aero_force_x,aero_force_y,aero_force_z,gravity_force_x,gravity_force_y,gravity_force_z,thrust_force_x,thrust_force_y,thrust_force_z,elevator,aileron,rudder,power_lever\n";
            let _ = file.write_all(header.as_bytes());
        }
        
        Self {
            time_steps: 0,
            max_steps: 1000, // Run for 10 seconds at 100Hz
            data_points: Vec::with_capacity(1000),
            log_file,
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

    // Configure physics
    let physics_config = PhysicsConfig::default();
    app.add_plugins((
        PhysicsPlugin::with_config(physics_config.clone()),
        TransformationPlugin::default(),
        EnvironmentPlugin::new(),
    ));

    // Add physics tracker
    app.insert_resource(PhysicsTracker::default());

    // Register systems
    app.add_systems(
        Update,
        (
            // Physics systems in order
            air_data_system,
            aero_force_system,
            force_calculator_system,
            physics_integrator_system,
            
            // Our tracking system
            track_physics.after(physics_integrator_system),
        ),
    );

    // Spawn aircraft entity
    app.add_systems(Startup, setup_aircraft);

    // Run the app
    println!("Starting C172 physics test simulation...");
    app.run();
}

// System to set up the aircraft
fn setup_aircraft(mut commands: Commands) {
    // Create an aircraft with initial state for 55 m/s level flight
    let entity = commands
        .spawn((
            // Spatial component with initial velocity (55 m/s along X axis)
            SpatialComponent {
                position: Vector3::new(0.0, 1000.0, 0.0), // 1000m altitude
                velocity: Vector3::new(55.0, 0.0, 0.0),   // 55 m/s along X axis
                attitude: UnitQuaternion::from_euler_angles(0.0, 3.0f64.to_radians(), 0.0), // 3° pitch
                angular_velocity: Vector3::zeros(),
            },
            // Initial control surfaces
            AircraftControlSurfaces {
                elevator: -0.1,  // Slightly nose-down elevator
                aileron: 0.0,    // Neutral aileron
                rudder: 0.0,     // Neutral rudder
                power_lever: 0.3, // 30% throttle
            },
            // Propulsion state with engine running
            PropulsionState::new(1),
            // Default air data
            AirData::default(),
            // Use Cessna 172 configuration
            FullAircraftConfig::cessna172(),
        ))
        .id();

    println!("Cessna 172 entity {:?} created at 1000m altitude with 55 m/s airspeed", entity);
    println!("Initial attitude: 3° pitch up, level wings");
    println!("Initial controls: elevator=-0.1, power=0.3 (30%)");
    println!("Physics test will run for 10 seconds (1000 steps)");
    println!("Data will be logged to 'c172_physics_log.csv'");
}

// System to track physics at each time step
fn track_physics(
    time: Res<Time>,
    mut tracker: ResMut<PhysicsTracker>,
    query: Query<(
        &SpatialComponent, 
        &AircraftControlSurfaces, 
        &AirData, 
        &PhysicsComponent
    )>,
) {
    // Check if we've reached max steps
    if tracker.time_steps >= tracker.max_steps {
        // We're done with the simulation
        std::process::exit(0);
    }
    
    for (spatial, controls, air_data, physics) in query.iter() {
        // Extract roll, pitch, yaw angles in degrees
        let (roll, pitch, yaw) = spatial.attitude.euler_angles();
        let attitude = (
            roll.to_degrees(), 
            pitch.to_degrees(), 
            yaw.to_degrees()
        );
        
        // Extract alpha, beta in degrees
        let alpha = air_data.alpha.to_degrees();
        let beta = air_data.beta.to_degrees();
        
        // Extract control values
        let controls_tuple = (
            controls.elevator,
            controls.aileron,
            controls.rudder,
            controls.power_lever,
        );
        
        // Extract forces from the physics component
        let net_force = physics.net_force;
        let net_moment = physics.net_moment;
        
        // Calculate components of forces by category
        let mut aero_force = Vector3::zeros();
        let mut gravity_force = Vector3::zeros();
        let mut thrust_force = Vector3::zeros();
        
        for force in &physics.forces {
            match force.category {
                ForceCategory::Aerodynamic => aero_force += force.vector,
                ForceCategory::Gravitational => gravity_force += force.vector,
                ForceCategory::Propulsion => thrust_force += force.vector,
                _ => {} // Ignore other categories
            }
        }
        
        // Create data point
        let data_point = PhysicsDataPoint {
            time: time.elapsed_seconds(),
            position: spatial.position,
            velocity: spatial.velocity,
            attitude,
            alpha,
            beta,
            forces: net_force,
            moments: net_moment,
            total_aero_force: aero_force,
            gravity_force,
            thrust_force,
            controls: controls_tuple,
        };
        
        // Save data point
        tracker.data_points.push(data_point.clone());
        
        // Log data to file
        if let Some(ref mut file) = tracker.log_file {
            let data_str = format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                data_point.time,
                data_point.position.x, data_point.position.y, data_point.position.z,
                data_point.velocity.x, data_point.velocity.y, data_point.velocity.z,
                data_point.attitude.0, data_point.attitude.1, data_point.attitude.2,
                data_point.alpha, data_point.beta,
                data_point.forces.x, data_point.forces.y, data_point.forces.z,
                data_point.moments.x, data_point.moments.y, data_point.moments.z,
                data_point.total_aero_force.x, data_point.total_aero_force.y, data_point.total_aero_force.z,
                data_point.gravity_force.x, data_point.gravity_force.y, data_point.gravity_force.z,
                data_point.thrust_force.x, data_point.thrust_force.y, data_point.thrust_force.z,
                data_point.controls.0, data_point.controls.1, data_point.controls.2, data_point.controls.3
            );
            let _ = file.write_all(data_str.as_bytes());
        }
        
        // Print progress every 100 steps
        if tracker.time_steps % 100 == 0 {
            println!("Time step {}: t={:.2}s", tracker.time_steps, data_point.time);
            println!("  Position: [{:.1}, {:.1}, {:.1}] m", 
                     data_point.position.x, data_point.position.y, data_point.position.z);
            println!("  Velocity: [{:.1}, {:.1}, {:.1}] m/s, |V|={:.1} m/s", 
                     data_point.velocity.x, data_point.velocity.y, data_point.velocity.z,
                     data_point.velocity.norm());
            println!("  Attitude: roll={:.1}°, pitch={:.1}°, yaw={:.1}°", 
                     data_point.attitude.0, data_point.attitude.1, data_point.attitude.2);
            println!("  α={:.2}°, β={:.2}°", data_point.alpha, data_point.beta);
            println!("  Controls: elevator={:.2}, aileron={:.2}, rudder={:.2}, power={:.2}", 
                     data_point.controls.0, data_point.controls.1, data_point.controls.2, data_point.controls.3);
            println!("  Net force: [{:.1}, {:.1}, {:.1}] N", 
                     data_point.forces.x, data_point.forces.y, data_point.forces.z);
            println!("  Net moment: [{:.1}, {:.1}, {:.1}] N·m", 
                     data_point.moments.x, data_point.moments.y, data_point.moments.z);
            println!("  Force components:");
            println!("    Aero:    [{:.1}, {:.1}, {:.1}] N", 
                     data_point.total_aero_force.x, data_point.total_aero_force.y, data_point.total_aero_force.z);
            println!("    Gravity: [{:.1}, {:.1}, {:.1}] N", 
                     data_point.gravity_force.x, data_point.gravity_force.y, data_point.gravity_force.z);
            println!("    Thrust:  [{:.1}, {:.1}, {:.1}] N", 
                     data_point.thrust_force.x, data_point.thrust_force.y, data_point.thrust_force.z);
            println!("");
        }
    }
    
    // Increment time step counter
    tracker.time_steps += 1;
}