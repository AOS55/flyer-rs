use bevy::{app::AppExit, prelude::*};
use flyer::{
    components::{
        AirData, AircraftControlSurfaces, FixedStartConfig, ForceCategory, FullAircraftConfig,
        NeedsTrim, PhysicsComponent, SpatialComponent, StartConfig, TrimCondition, TrimRequest,
        TrimSolverConfig,
    },
    plugins::{
        // Removed TrimPlugin from here
        EnvironmentPlugin,
        FullAircraftPlugin,
        PhysicsPlugin,
        StartupSequencePlugin,
        TransformationPlugin,
    },
    resources::PhysicsConfig,
    systems::{
        // Core physics systems
        aero_force_system,
        air_data_system,
        force_calculator_system,
        // Trim systems - Ensure these paths are correct for your library structure!
        handle_trim_requests, // Handler system
        physics_integrator_system,
        propulsion_system,
        trim_aircraft_system, // Main trim logic system
    },
};
use nalgebra::Vector3;
use std::fs::File;
use std::io::Write;

// --- Resources (SimulationTimer, StateTracker - unchanged) ---

#[derive(Resource)]
struct SimulationTimer {
    max_time: f64,
    exit_after_log: bool,
}

#[derive(Resource)]
struct StateTracker {
    data_points: Vec<StateDataPoint>,
    log_file_path: String,
    time_steps: usize,
}

// --- Components (TrackedAircraft - unchanged) ---

#[derive(Component)]
struct TrackedAircraft;

// --- Data Structures (StateDataPoint, StateTracker impl - unchanged) ---

#[derive(Clone, Debug)]
struct StateDataPoint {
    time: f64,
    pos: Vector3<f64>,
    vel: Vector3<f64>,
    att: (f64, f64, f64),     // Roll, pitch, yaw (deg)
    rate: (f64, f64, f64),    // p, q, r (deg/s)
    alpha: f64,               // deg
    beta: f64,                // deg
    net_force: Vector3<f64>,  // Inertial
    net_moment: Vector3<f64>, // Body
    aero_f_body: Vector3<f64>,
    prop_f_body: Vector3<f64>,
    grav_f_body: Vector3<f64>,
    ctrl: (f64, f64, f64, f64), // elev, ail, rud, pwr
}

impl StateTracker {
    fn new(log_file_path: &str) -> Self {
        Self {
            data_points: Vec::new(),
            log_file_path: log_file_path.to_string(),
            time_steps: 0,
        }
    }

    fn write_log(&self) {
        if let Ok(mut file) = File::create(&self.log_file_path) {
            // CSV Header
            let header = "time,pos_x,pos_y,pos_z,vel_x,vel_y,vel_z,roll_deg,pitch_deg,yaw_deg,roll_rate_deg_s,pitch_rate_deg_s,yaw_rate_deg_s,p_deg_s,q_deg_s,r_deg_s,alpha_deg,beta_deg,net_Fx_inertial,net_Fy_inertial,net_Fz_inertial,net_Mx_body,net_My_body,net_Mz_body,aero_x,aero_y,aero_z,aero_Fx_body,aero_Fy_body,aero_Fz_body,prop_Fx_body,prop_Fy_body,prop_Fz_body,grav_Fx_body,grav_Fy_body,grav_Fz_body,elevator,aileron,rudder,power\n";
            let _ = file.write_all(header.as_bytes());
            // Write data points
            for dp in &self.data_points {
                let data_str = format!(
                    "{:.4},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.3},{:.3},{:.3},{:.3}\n",
                    dp.time, dp.pos.x, dp.pos.y, dp.pos.z, dp.vel.x, dp.vel.y, dp.vel.z, dp.att.0, dp.att.1, dp.att.2,
                    // Added roll_rate_deg_s, pitch_rate_deg_s, yaw_rate_deg_s (same as p_deg_s, q_deg_s, r_deg_s)
                    dp.rate.0, dp.rate.1, dp.rate.2,
                    // Original p_deg_s, q_deg_s, r_deg_s
                    dp.rate.0, dp.rate.1, dp.rate.2,
                    dp.alpha, dp.beta, dp.net_force.x, dp.net_force.y, dp.net_force.z,
                    dp.net_moment.x, dp.net_moment.y, dp.net_moment.z,
                    // Added aero_x, aero_y, aero_z (same as aero_Fx_body, etc.)
                    dp.aero_f_body.x, dp.aero_f_body.y, dp.aero_f_body.z,
                    // Original aero forces
                    dp.aero_f_body.x, dp.aero_f_body.y, dp.aero_f_body.z,
                    dp.prop_f_body.x, dp.prop_f_body.y, dp.prop_f_body.z,
                    dp.grav_f_body.x, dp.grav_f_body.y, dp.grav_f_body.z,
                    dp.ctrl.0, dp.ctrl.1, dp.ctrl.2, dp.ctrl.3
                );
                let _ = file.write_all(data_str.as_bytes());
            }
            println!("State log written to '{}'", self.log_file_path);
        } else {
            eprintln!("Error creating log file '{}'", self.log_file_path);
        }
    }
}

fn main() {
    let mut app = App::new();

    let simulation_time = 100.0;
    let physics_hz = 100.0;

    // Minimal Bevy setup
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Time::<Fixed>::from_hz(physics_hz));

    // --- Flyer Library Setup ---
    app.insert_resource(TrimSolverConfig::default());

    // *** Manually add Trim Event instead of using TrimPlugin ***
    app.add_event::<TrimRequest>();

    app.add_plugins((
        EnvironmentPlugin::new(),
        PhysicsPlugin::with_config(PhysicsConfig {
            timestep: 1.0 / physics_hz,
            ..default()
        }),
        StartupSequencePlugin,
        TransformationPlugin::default(),
        // TrimPlugin, // REMOVED TrimPlugin
    ));

    // --- Aircraft Configuration ---
    let aircraft_config_data = FullAircraftConfig::f16c();
    let initial_speed = 150.0;
    let initial_altitude_m = 500.0;
    let start_config = StartConfig::Fixed(FixedStartConfig {
        position: Vector3::new(0.0, 0.0, -initial_altitude_m),
        speed: initial_speed,
        heading: 0.0,
    });
    let mut aircraft_plugin_config = aircraft_config_data.clone();
    aircraft_plugin_config.start_config = start_config;
    app.add_plugins(FullAircraftPlugin::new_single(aircraft_plugin_config));

    // --- Simulation Control & Logging ---
    app.insert_resource(SimulationTimer {
        max_time: simulation_time,
        exit_after_log: true,
    });
    app.insert_resource(StateTracker::new("trim_test_manual_log.csv")); // Changed log name

    // --- System Scheduling ---
    app.add_systems(
        FixedUpdate,
        (
            update_aircraft_controls,
            // Core physics loop
            air_data_system,
            aero_force_system,
            propulsion_system,
            force_calculator_system,
            physics_integrator_system,
            // Main trim logic system (added manually)
            trim_aircraft_system, // Ensure path is correct
            // Log state
            track_aircraft_state,
        )
            .chain(),
    );

    app.add_systems(
        Update,
        (
            // *** Manually add trim event handler instead of using TrimPlugin ***
            handle_trim_requests, // Ensure path is correct
            check_simulation_time,
            write_log_on_exit.run_if(on_event::<AppExit>),
        )
            .chain(),
    );

    app.add_systems(PostStartup, setup_tracking_and_trigger_trim);

    // --- Run ---
    println!("Starting Trim Test Simulation (Manual Setup)...");
    println!("Target: Straight & Level Flight @ {:.1} m/s", initial_speed);
    println!("Duration: {:.1} seconds", simulation_time);
    println!("Log will be saved to 'trim_test_manual_log.csv'");
    app.run();
}

// --- Systems (setup_tracking_and_trigger_trim, update_aircraft_controls, track_aircraft_state, check_simulation_time, write_log_on_exit) ---
// These systems remain the same as the previous correct version. Make sure they are included here.

/// System to add tracking marker and send the initial trim request.
fn setup_tracking_and_trigger_trim(
    mut commands: Commands,
    mut trim_requests: EventWriter<TrimRequest>,
    query: Query<(Entity, &FullAircraftConfig), Without<TrackedAircraft>>,
) {
    if let Ok((entity, aircraft_config)) = query.get_single() {
        println!("Adding TrackedAircraft marker to {:?}", entity);
        commands.entity(entity).insert(TrackedAircraft);

        let target_airspeed = match &aircraft_config.start_config {
            StartConfig::Fixed(fsc) => fsc.speed,
            _ => {
                warn!("Aircraft config has no fixed start speed, using 80.0 m/s");
                80.0
            }
        };

        println!(
            "Sending TrimRequest for StraightLevel @ {:.1} m/s to {:?}",
            target_airspeed, entity
        );
        trim_requests.send(TrimRequest {
            entity,
            condition: TrimCondition::StraightAndLevel {
                airspeed: target_airspeed,
            },
        });
    } else {
        warn!("Could not find unique aircraft entity to start trim.");
    }
}

/// Updates aircraft controls based on time. Holds trim initially, then pulses.
fn update_aircraft_controls(
    time: Res<Time>,
    // Query aircraft that have the marker AND are NOT currently being trimmed
    // This prevents interfering while the trim system is actively running (if it takes multiple frames)
    // Or, more simply, just use time checks if trim is expected to finish quickly.
    mut query: Query<&mut AircraftControlSurfaces, (With<TrackedAircraft>, Without<NeedsTrim>)>,
    // If trim finishes in one frame, you might not need Without<NeedsTrim>
    // mut query: Query<&mut AircraftControlSurfaces, With<TrackedAircraft>>,
) {
    let current_time = time.elapsed_secs_f64();
    // Increase this hold duration to give trim time and observe steady flight
    // let trim_hold_duration = 15.0; // Hold trimmed controls for 15 seconds
    let pulse_duration = 1.0;
    let pulse_start_time = 15.0; // Start pulses after hold

    for mut controls in query.iter_mut() {
        // --- Let Trim System Dictate Initial Controls ---
        // DO NOT set default values like:
        // The trim_aircraft_system's call to apply_trim_state will set the correct
        // initial elevator and power_lever. We only modify them later for testing.

        // --- Apply Example Pulses AFTER Trim Hold Period ---
        if current_time >= pulse_start_time {
            // Elevator pulse example
            if current_time < (pulse_start_time + pulse_duration) {
                // Apply a CHANGE relative to the trimmed value, not overwrite it
                controls.elevator += 0.05;
                // Or set to an absolute value if desired: controls.elevator = 0.1;
                info!(
                    "Applying elevator pulse relative to trim at t={:.2}s",
                    current_time
                );
            }
            // Add other pulses similarly, e.g., for aileron/rudder at later times
            else if current_time >= (pulse_start_time + 5.0)
                && current_time < (pulse_start_time + 5.0 + pulse_duration)
            {
                controls.aileron = 0.1; // Set absolute aileron for a roll test
                info!("Applying aileron pulse at t={:.2}s", current_time);
            } else {
                // Reset pulsed controls AFTER the pulse duration if needed
                // (otherwise they stay applied)
                // Example: Reset aileron after its pulse
                if current_time >= (pulse_start_time + 5.0 + pulse_duration)
                    && controls.aileron != 0.0
                {
                    controls.aileron = 0.0;
                }
                // Don't reset elevator if the pulse was additive, or reset it carefully
            }
        }
    }
}

/// System to track aircraft state at each time step and log to CSV
fn track_aircraft_state(
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
    time: Res<Time>,
    physics_config: Res<PhysicsConfig>,
) {
    if let Ok((spatial, controls, air_data, physics)) = query.get_single() {
        tracker.time_steps += 1;
        // ... state extraction logic ...
        let (roll, pitch, yaw) = spatial.attitude.euler_angles();
        let attitude_deg = (roll.to_degrees(), pitch.to_degrees(), yaw.to_degrees());
        let rates_deg_s = (
            spatial.angular_velocity.x.to_degrees(),
            spatial.angular_velocity.y.to_degrees(),
            spatial.angular_velocity.z.to_degrees(),
        );
        let mut aero_f_body = Vector3::zeros();
        let mut prop_f_body = Vector3::zeros();
        let mut grav_f_body = Vector3::zeros();
        for force in &physics.forces {
            match force.category {
                ForceCategory::Aerodynamic => aero_f_body += force.vector,
                ForceCategory::Propulsive => prop_f_body += force.vector,
                ForceCategory::Gravitational => grav_f_body += force.vector,
                _ => {}
            }
        }
        let data_point = StateDataPoint {
            time: time.elapsed_secs_f64(),
            pos: spatial.position,
            vel: spatial.velocity,
            att: attitude_deg,
            rate: rates_deg_s,
            alpha: air_data.alpha.to_degrees(),
            beta: air_data.beta.to_degrees(),
            net_force: physics.net_force,
            net_moment: physics.net_moment,
            aero_f_body,
            prop_f_body,
            grav_f_body,
            ctrl: (
                controls.elevator,
                controls.aileron,
                controls.rudder,
                controls.power_lever,
            ),
        };
        tracker.data_points.push(data_point);

        // Optional: Print status periodically
        let log_interval_steps = (2.0 / physics_config.timestep).round() as usize;
        if log_interval_steps > 0 && tracker.time_steps % log_interval_steps == 0 {
            println!( "t={:.1}s | Alt={:.1}m | V={:.1}m/s | Pitch={:.1}deg | Alpha={:.1}deg | Elev={:.9} | Pwr={:.9}", time.elapsed_secs_f64(), -spatial.position.z, spatial.velocity.norm(), attitude_deg.1, air_data.alpha.to_degrees(), controls.elevator, controls.power_lever );
        }
    }
}

/// System to check simulation time and trigger exit
fn check_simulation_time(
    time: Res<Time>,
    timer: Res<SimulationTimer>,
    mut exit: EventWriter<AppExit>,
    mut logged_and_exiting: Local<bool>,
) {
    if *logged_and_exiting {
        return;
    }
    if time.elapsed_secs_f64() >= timer.max_time {
        info!(
            "Simulation complete: Reached max time {:.1}s.",
            timer.max_time
        );
        if timer.exit_after_log {
            info!("Triggering AppExit.");
            *logged_and_exiting = true;
            exit.send(AppExit::Success);
        }
    }
}

// System that runs when AppExit is triggered
fn write_log_on_exit(tracker: Res<StateTracker>) {
    info!("App exiting, writing final log...");
    tracker.write_log();
    info!("Log writing complete.");
}
