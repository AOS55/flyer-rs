use bevy::prelude::*;
use flyer::{
    components::{
        FixedStartConfig, FullAircraftConfig, LateralBounds, LongitudinalBounds, NeedsTrim,
        SpatialComponent, StartConfig, TrimCondition, TrimRequest, TrimSolverConfig,
    },
    plugins::{EnvironmentPlugin, FullAircraftPlugin, PhysicsPlugin, TransformationPlugin},
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, handle_trim_requests,
        physics_integrator_system, trim_aircraft_system,
    },
};
use nalgebra::Vector3;
use std::{fs::OpenOptions, io::Write, path::PathBuf, time::Instant};

#[derive(Resource)]
struct TrimConvergenceTracker {
    last_cost: f64,
    stall_counter: u32,
    iterations: u32,
}

impl Default for TrimConvergenceTracker {
    fn default() -> Self {
        Self {
            last_cost: f64::INFINITY,
            stall_counter: 0,
            iterations: 0,
        }
    }
}

#[derive(Debug)]
struct TrimStateLog {
    iteration: u32,
    cost: f64,
    airspeed: f64,
    altitude: f64,
    // Longitudinal states
    alpha: f64,
    pitch: f64,
    elevator: f64,
    throttle: f64,
}

#[derive(Resource)]
pub struct TrimLogger {
    start_time: Instant,
    log_file: PathBuf,
    initialized: bool,
}

impl Default for TrimLogger {
    fn default() -> Self {
        let log_file = std::env::temp_dir().join("trim_convergence.csv");
        println!("Trim log file will be created at: {:?}", log_file);

        Self {
            start_time: Instant::now(),
            log_file,
            initialized: false,
        }
    }
}

impl TrimLogger {
    fn initialize(&mut self) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.log_file)?;

        writeln!(
            file,
            "timestamp,iteration,cost,airspeed,altitude,alpha,pitch,elevator,throttle"
        )?;

        self.initialized = true;
        Ok(())
    }

    fn log_state(&self, record: &TrimStateLog) -> std::io::Result<()> {
        let mut file = OpenOptions::new().append(true).open(&self.log_file)?;
        let timestamp = self.start_time.elapsed().as_secs_f64();

        writeln!(
            file,
            "{:.6},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            timestamp,
            record.iteration,
            record.cost,
            record.airspeed,
            record.altitude,
            record.alpha,
            record.pitch,
            record.elevator,
            record.throttle,
        )
    }
}

fn main() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default())
        .init_asset::<Image>()
        .init_resource::<Assets<TextureAtlasLayout>>();
    app.insert_resource(Time::<Fixed>::from_hz(1e6));

    app.add_event::<TrimRequest>();

    // Add required plugins
    app.add_plugins((
        PhysicsPlugin::with_config(PhysicsConfig::default()),
        TransformationPlugin::default(),
        EnvironmentPlugin::new(),
    ));

    app.insert_resource(TrimConvergenceTracker::default());
    app.insert_resource(TrimLogger::default());

    // Configure trim solver
    app.insert_resource(TrimSolverConfig {
        max_iterations: 10000,
        cost_tolerance: 1e-2,
        use_gradient_refinement: true,
        lateral_bounds: LateralBounds::default(),
        longitudinal_bounds: LongitudinalBounds::default(),
    });

    // Create basic aircraft config
    let aircraft_config = {
        let mut config = FullAircraftConfig::default();
        config.start_config = StartConfig::Fixed(FixedStartConfig {
            position: Vector3::new(0.0, 0.0, -1000.0),
            speed: 100.0,
            heading: 0.0,
        });
        config
    };

    app.add_plugins(FullAircraftPlugin::new_single(aircraft_config));

    // Add systems
    app.add_systems(
        FixedUpdate,
        (
            air_data_system,
            aero_force_system,
            force_calculator_system,
            physics_integrator_system,
            request_trim,
            handle_trim_requests,
            trim_aircraft_system,
            monitor_trim_convergence,
        )
            .chain(),
    );

    app.run();
}

fn request_trim(
    mut trim_requests: EventWriter<TrimRequest>,
    query: Query<(Entity, &SpatialComponent), Added<SpatialComponent>>,
) {
    for (entity, spatial) in query.iter() {
        println!(
            "Requesting trim at altitude: {} meters",
            -spatial.position.z
        );

        trim_requests.send(TrimRequest {
            entity,
            condition: TrimCondition::StraightAndLevel { airspeed: 100.0 },
        });
    }
}

fn monitor_trim_convergence(
    query: Query<(&SpatialComponent, Option<&NeedsTrim>)>,
    mut tracker: ResMut<TrimConvergenceTracker>,
    mut trim_logger: ResMut<TrimLogger>,
    time: Res<Time>,
) {
    if !trim_logger.initialized {
        if let Err(e) = trim_logger.initialize() {
            eprintln!("Failed to initialize trim logger: {}", e);
            return;
        }
    }

    for (spatial, needs_trim) in query.iter() {
        if let Some(needs_trim) = needs_trim {
            if let Some(ref solver) = needs_trim.solver {
                let current_cost = solver.best_cost;
                tracker.iterations += 1;

                if !current_cost.is_finite() {
                    println!(
                        "Warning: Cost became non-finite at iteration {}",
                        tracker.iterations
                    );
                    return;
                }

                let state = &solver.current_state;
                let trim_state = TrimStateLog {
                    iteration: tracker.iterations,
                    cost: current_cost,
                    airspeed: spatial.velocity.norm(),
                    altitude: -spatial.position.z,
                    alpha: state.longitudinal.alpha,
                    pitch: state.longitudinal.theta,
                    elevator: state.longitudinal.elevator,
                    throttle: state.longitudinal.power_lever,
                };

                if let Err(e) = trim_logger.log_state(&trim_state) {
                    eprintln!("Failed to log trim state: {}", e);
                }

                println!(
                    "Iteration {}: Cost = {:.6}\nLongitudinal: alpha = {:.1}°, theta = {:.1}°, elevator = {:.3}, throttle = {:.3}",
                    tracker.iterations,
                    current_cost,
                    trim_state.alpha.to_degrees(),
                    trim_state.pitch.to_degrees(),
                    trim_state.elevator,
                    trim_state.throttle,
                );

                // Check for stall
                if (tracker.last_cost - current_cost).abs() < 1e-6 {
                    tracker.stall_counter += 1;
                    if tracker.stall_counter > 5 {
                        println!("Optimization stalled - not making progress");
                        return;
                    }
                } else {
                    tracker.stall_counter = 0;
                }

                tracker.last_cost = current_cost;
            }
        } else if tracker.iterations > 0 {
            // Print completion
            let (_, pitch, _) = spatial.attitude.euler_angles();
            println!("\nTrim complete!");
            println!(
                "Final state: Speed = {:.1} m/s, Alt = {:.1}m, Pitch = {:.1}°",
                spatial.velocity.norm(),
                -spatial.position.z,
                pitch.to_degrees(),
            );
            tracker.iterations = 0;
            return;
        }
    }
}
