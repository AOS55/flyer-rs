use bevy::prelude::*;
use flyer::{
    components::{
        AirData, AircraftControlSurfaces, FixedStartConfig, ForceCategory, FullAircraftConfig,
        PhysicsComponent, SpatialComponent, StartConfig,
    },
    plugins::{
        EnvironmentPlugin, FullAircraftPlugin, PhysicsPlugin, StartupSequencePlugin,
        TransformationPlugin,
    },
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, physics_integrator_system,
        propulsion_system,
    },
};
use nalgebra::Vector3;
use std::fs::File;
use std::io::Write;

// Resource to track simulation time for termination
#[derive(Resource)]
struct SimulationTimer {
    max_time: f64, // Maximum simulation time in seconds
}

// Define a component marker for the aircraft we want to track
#[derive(Component)]
struct TrackedAircraft;

// Define a struct to hold a single data point of tracked state
#[derive(Clone, Debug)]
struct StateDataPoint {
    time: f64,
    position: Vector3<f64>,
    velocity: Vector3<f64>,
    attitude: (f64, f64, f64), // Roll, pitch, yaw in degrees
    rate: (f64, f64, f64),     // Roll rate, pitch rate, yaw rate in degrees per second
    alpha: f64,                // Angle of attack in degrees
    beta: f64,                 // Sideslip angle in degrees
    forces: Vector3<f64>,
    moments: Vector3<f64>,
    total_aero_force: Vector3<f64>,
    gravity_force: Vector3<f64>,
    thrust_force: Vector3<f64>,
    controls: (f64, f64, f64, f64), // Elevator, aileron, rudder, power
}

// Define a resource to store all tracking data
#[derive(Resource)]
struct StateTracker {
    data_points: Vec<StateDataPoint>,
    log_file: Option<File>,
    time_steps: usize,
}

impl Default for StateTracker {
    fn default() -> Self {
        // Create log file
        let mut log_file = File::create("aircraft_state_log.csv").ok();

        // Write headers to log file
        if let Some(ref mut file) = log_file {
            let header = "time,pos_x,pos_y,pos_z,vel_x,vel_y,vel_z,roll_deg,pitch_deg,yaw_deg,roll_rate_deg_s,pitch_rate_deg_s,yaw_rate_deg_s,alpha_deg,beta_deg,force_x,force_y,force_z,moment_x,moment_y,moment_z,aero_x,aero_y,aero_z,gravity_x,gravity_y,gravity_z,thrust_x,thrust_y,thrust_z,elevator,aileron,rudder,power\n";
            let _ = file.write_all(header.as_bytes());
        }

        Self {
            data_points: Vec::new(),
            log_file,
            time_steps: 0,
        }
    }
}

fn main() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Time::<Fixed>::from_hz(1e3));

    // Add termination resource - set to run for 100 seconds
    app.insert_resource(SimulationTimer { max_time: 100.0 });

    // Add state tracking resource
    app.init_resource::<StateTracker>();

    // Add required plugins
    app.add_plugins((
        StartupSequencePlugin,
        PhysicsPlugin::with_config(PhysicsConfig {
            timestep: 1.0 / 1e3, // 100Hz physics timestep
            ..default()
        }),
    ));

    // Create TwinOtter configuration
    let mut twin_otter_config = FullAircraftConfig::twin_otter();

    // Set up a fixed start state at 1000m up, at x,y (0,0), 55 m/s and 0 heading
    twin_otter_config.start_config = StartConfig::Fixed(FixedStartConfig {
        position: Vector3::new(0.0, 0.0, -1000.0), // z is negative for altitude
        speed: 80.0,                               // m/s
        heading: 0.0,                              // degrees
    });

    app.add_plugins((TransformationPlugin::new(1.0), EnvironmentPlugin::new()));

    // Add aircraft plugin with config
    app.add_plugins(FullAircraftPlugin::new_single(twin_otter_config));

    // Add physics update system
    app.add_systems(
        FixedUpdate,
        (
            air_data_system,
            aero_force_system,
            force_calculator_system,
            propulsion_system,
            physics_integrator_system,
            update_aircraft_controls,
        )
            .chain(),
    );

    // This ensures all required components are added to the entity first
    app.add_systems(PostStartup, setup_tracking);

    // Add state tracking system
    app.add_systems(FixedUpdate, track_aircraft_state);

    // Add termination system to check if simulation should end
    app.add_systems(Update, check_simulation_time);

    // Run the app
    println!("Starting C172 physics test simulation...");
    println!("Physics test will run for 10 seconds (1000 steps)");
    println!("Data will be logged to 'aircraft_state_log.csv'");
    app.run();
}

// System to check if simulation time has reached the maximum and exit if so
fn check_simulation_time(time: Res<Time>, timer: Res<SimulationTimer>) {
    if time.elapsed_secs_f64() >= timer.max_time {
        println!(
            "Simulation complete: reached maximum time of {} seconds",
            timer.max_time
        );
        std::process::exit(0);
    }
}

// System to add tracking marker to the aircraft
fn setup_tracking(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &SpatialComponent,
            &AircraftControlSurfaces,
            &AirData,
        ),
        Without<TrackedAircraft>,
    >,
) {
    let count = query.iter().count();
    println!(
        "Found {} aircraft entities with all required components",
        count
    );

    // Only try to track entities that have ALL necessary components
    for (entity, spatial, controls, air_data) in query.iter() {
        commands.entity(entity).insert(TrackedAircraft);
        println!("Added tracking marker to aircraft entity: {:?}", entity);
        println!("  Initial position: {:?}", spatial.position);
        println!(
            "  Initial controls: elevator={:.2}, aileron={:.2}, rudder={:.2}, power={:.2}",
            controls.elevator, controls.aileron, controls.rudder, controls.power_lever
        );
        println!(
            "  Initial alpha: {:.2}°, beta: {:.2}°",
            air_data.alpha.to_degrees(),
            air_data.beta.to_degrees()
        );
    }

    // If no entities found, print a warning
    if count == 0 {
        println!("WARNING: No aircraft entities found with all required components!");
        println!("Check that your aircraft has been properly initialized");
    }
}

fn update_aircraft_controls(
    time: Res<Time>,
    mut query: Query<&mut AircraftControlSurfaces, With<TrackedAircraft>>,
) {
    let current_time = time.elapsed_secs_f64();

    for mut controls in query.iter_mut() {
        // Default control values
        controls.elevator = 0.0;
        controls.aileron = 0.0;
        controls.rudder = 0.0;
        controls.power_lever = 0.27;

        // Pulse duration (in seconds)
        let pulse_duration = 2.0;

        // Aileron pulse at 20s
        if current_time >= 20.0 && current_time < (20.0 + pulse_duration) {
            controls.aileron = 0.1; // positive aileron deflection
            println!("Applying aileron pulse at t={:.2}s", current_time);
        }

        // Rudder pulse at 30s
        if current_time >= 30.0 && current_time < (30.0 + pulse_duration) {
            controls.rudder = 0.1; // positive rudder deflection
            println!("Applying rudder pulse at t={:.2}s", current_time);
        }

        // Elevator pulse at 40s
        if current_time >= 40.0 && current_time < (40.0 + pulse_duration) {
            controls.elevator = 0.1; // positive elevator deflection
            println!("Applying elevator pulse at t={:.2}s", current_time);
        }
    }
}

// System to track aircraft state at each time step
fn track_aircraft_state(
    time: Res<Time>,
    mut tracker: ResMut<StateTracker>,
    query: Query<
        (
            &SpatialComponent,
            &AircraftControlSurfaces,
            &AirData,
            &PhysicsComponent,
        ),
        With<TrackedAircraft>,
    >,
) {
    tracker.time_steps += 1;

    for (spatial, controls, air_data, physics) in query.iter() {
        // Extract roll, pitch, yaw angles in degrees
        let (roll, pitch, yaw) = spatial.attitude.euler_angles();
        let attitude = (roll.to_degrees(), pitch.to_degrees(), yaw.to_degrees());

        // Extract angular velocity and convert to tuple of degrees/second
        let ang_vel = (
            spatial.angular_velocity[0].to_degrees(),
            spatial.angular_velocity[1].to_degrees(),
            spatial.angular_velocity[2].to_degrees(),
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
        let mut aero_force = Vector3::<f64>::zeros();
        let mut gravity_force = Vector3::<f64>::zeros();
        let mut thrust_force = Vector3::<f64>::zeros();

        for force in &physics.forces {
            match force.category {
                ForceCategory::Aerodynamic => aero_force += force.vector,
                ForceCategory::Gravitational => gravity_force += force.vector,
                ForceCategory::Propulsive => thrust_force += force.vector,
                _ => {} // Ignore other categories
            }
        }

        // Create data point
        let data_point = StateDataPoint {
            time: time.elapsed_secs_f64(),
            position: spatial.position,
            velocity: spatial.velocity,
            rate: ang_vel,
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
        if let Some(ref mut file) = tracker.log_file.as_mut() {
            let data_str = format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                data_point.time,
                data_point.position.x, data_point.position.y, data_point.position.z,
                data_point.velocity.x, data_point.velocity.y, data_point.velocity.z,
                data_point.attitude.0, data_point.attitude.1, data_point.attitude.2,
                data_point.rate.0, data_point.rate.1, data_point.rate.2,
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

        if tracker.time_steps % 100 == 0 {
            println!(
                "t={:.2}s, u={:.2}, w={:.2}, alpha={:.2}°, X_aero={:.1}",
                data_point.time,
                data_point.velocity.x,
                data_point.velocity.z,
                data_point.alpha,
                data_point.total_aero_force.x
            );
        }

        // Print progress every 100 steps
        if tracker.time_steps % 100 == 0 {
            println!(
                "Time step {}: t={:.2}s",
                tracker.time_steps, data_point.time
            );
            println!(
                "  Position: [{:.1}, {:.1}, {:.1}] m",
                data_point.position.x, data_point.position.y, data_point.position.z
            );
            println!(
                "  Velocity: [{:.1}, {:.1}, {:.1}] m/s, |V|={:.1} m/s",
                data_point.velocity.x,
                data_point.velocity.y,
                data_point.velocity.z,
                data_point.velocity.norm()
            );
            println!(
                "  Attitude: roll={:.1}°, pitch={:.1}°, yaw={:.1}°",
                data_point.attitude.0, data_point.attitude.1, data_point.attitude.2
            );
            println!(
                "  Angular Rates: roll_rate={:.1}°/s, pitch_rate={:.1}°/s, yaw_rate={:.1}°/s",
                data_point.rate.0, data_point.rate.1, data_point.rate.2
            );
        }
    }
}
