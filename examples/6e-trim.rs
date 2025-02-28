use bevy::prelude::*;

use flyer::{
    components::{
        AirData, AircraftControlSurfaces, FixedStartConfig, FullAircraftConfig, PhysicsComponent,
        SpatialComponent, StartConfig,
    },
    plugins::{
        EnvironmentPlugin, FullAircraftPlugin, PhysicsPlugin, StartupSequencePlugin,
        TransformationPlugin,
    },
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, physics_integrator_system,
    },
};
use nalgebra::{UnitQuaternion, Vector3};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

#[derive(Resource)]
struct StateLogger {
    log_file: PathBuf,
    simulation_time: f64,
    dt: f64,
}

impl Default for StateLogger {
    fn default() -> Self {
        let log_file = std::env::temp_dir().join("aircraft_state_log.csv");
        // Create/overwrite the file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file)
        {
            writeln!(
                file,
                "time,vx,vy,vz,phi,theta,psi,alpha,beta,\
                 elevator,throttle,force_x,force_y,force_z,\
                 moment_x,moment_y,moment_z,\
                 airspeed,altitude"
            )
            .unwrap();
        }

        println!("State log file created at: {:?}", log_file);

        Self {
            log_file,
            simulation_time: 0.0,
            dt: 1.0 / 100.0, // Log at 100Hz
        }
    }
}

fn main() {
    let mut app = App::new();

    // Add minimal plugins without a window
    app.add_plugins(MinimalPlugins);

    // Set up fixed timestep physics
    let physics_dt = 1.0 / 100.0; // 1kHz physics
    app.insert_resource(Time::<Fixed>::from_seconds(physics_dt));
    // Add required plugins
    app.add_plugins((
        StartupSequencePlugin,
        PhysicsPlugin::with_config(PhysicsConfig {
            timestep: physics_dt,
            ..default()
        }),
        TransformationPlugin::new(1.0),
        EnvironmentPlugin::new(),
    ));

    // Create and add aircraft
    let mut aircraft_config = FullAircraftConfig::default();

    aircraft_config.start_config = StartConfig::Fixed(FixedStartConfig {
        position: Vector3::new(0.0, 0.0, -1000.0),
        speed: 70.0,
        heading: 0.0,
    });

    app.add_plugins(FullAircraftPlugin::new_single(aircraft_config));

    // Add physics systems
    app.add_systems(
        FixedUpdate,
        (
            air_data_system,
            aero_force_system,
            force_calculator_system,
            physics_integrator_system,
            set_controls,
        )
            .chain(),
    );

    // Add state logging
    app.insert_resource(StateLogger::default());
    app.add_systems(FixedUpdate, log_aircraft_state);

    // Set initial conditions
    app.add_systems(Startup, setup_initial_conditions);

    println!("Starting 30 second simulation...");
    let max_time = 100.0;
    while app.world_mut().resource::<StateLogger>().simulation_time < max_time {
        app.update();
    }
    println!("Simulation complete!");
}

fn setup_initial_conditions(mut query: Query<&mut AircraftControlSurfaces>) {
    if let Ok(mut controls) = query.get_single_mut() {
        // Initial control settings
        controls.elevator = -0.03;
        controls.power_lever = 1.0;
        controls.aileron = 0.0;
        controls.rudder = 0.0;
    }
}

fn set_controls(mut query: Query<&mut AircraftControlSurfaces>) {
    if let Ok(mut controls) = query.get_single_mut() {
        // Initial control settings
        controls.elevator = -0.03;
        controls.power_lever = 0.8;
        controls.aileron = 0.0;
        controls.rudder = 0.0;
    }
}

fn log_aircraft_state(
    mut logger: ResMut<StateLogger>,
    query: Query<(
        &SpatialComponent,
        &AircraftControlSurfaces,
        &AirData,
        &PhysicsComponent,
    )>,
) {
    if let Ok((spatial, controls, air_data, physics)) = query.get_single() {
        logger.simulation_time += logger.dt;

        // Get euler angles
        let (phi, theta, psi) = spatial.attitude.euler_angles();

        // Log state to file
        if let Ok(mut file) = OpenOptions::new().append(true).open(&logger.log_file) {
            writeln!(
                file,
                "{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},\
                 {:.3},{:.3},{:.3},{:.3},{:.3},\
                 {:.3},{:.3},{:.3},\
                 {:.3},{:.3}",
                logger.simulation_time,
                spatial.velocity.x,
                spatial.velocity.y,
                spatial.velocity.z,
                phi.to_degrees(),
                theta.to_degrees(),
                psi.to_degrees(),
                air_data.alpha.to_degrees(),
                air_data.beta.to_degrees(),
                controls.elevator,
                controls.power_lever,
                physics.net_force.x,
                physics.net_force.y,
                physics.net_force.z,
                physics.net_moment.x,
                physics.net_moment.y,
                physics.net_moment.z,
                air_data.true_airspeed,
                -spatial.position.z
            )
            .unwrap();
        }
    }
}
