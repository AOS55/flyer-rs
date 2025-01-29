use bevy::prelude::*;
use flyer::{
    components::{DubinsAircraftConfig, StartConfig},
    plugins::{DubinsAircraftPlugin, PhysicsPlugin, StartupSequencePlugin},
    resources::PhysicsConfig,
    systems::dubins_aircraft_system,
};
use nalgebra::Vector3;
use std::{fs::OpenOptions, io::Write, path::PathBuf, time::Instant};

// Resource to track elapsed times and compute metrics
#[derive(Resource)]
struct Metrics {
    elapsed_times: Vec<f64>,
    last_update: Instant,
    start_time: Instant,
    total_updates: u64,
    log_file: PathBuf,
}

impl Default for Metrics {
    fn default() -> Self {
        let log_file = std::env::temp_dir().join("dubins_aircraft_metrics.csv");
        // Create/overwrite the file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file)
        {
            writeln!(
                file,
                "timestamp,instant_rate,overall_rate,variance,total_updates"
            )
            .unwrap();
        }

        println!("Log file created at: {:?}", log_file);

        Self {
            elapsed_times: Vec::with_capacity(1000), // Pre-allocate for performance
            last_update: Instant::now(),
            start_time: Instant::now(),
            total_updates: 0,
            log_file,
        }
    }
}

impl Metrics {
    fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.elapsed_times.push(elapsed);
        self.total_updates += 1;
        self.last_update = now;

        // Keep last 1000 samples
        if self.elapsed_times.len() > 1000 {
            self.elapsed_times.remove(0);
        }
    }

    fn compute_stats(&self) -> (f64, f64, f64) {
        let mean = self.elapsed_times.iter().sum::<f64>() / self.elapsed_times.len() as f64;
        let variance = self
            .elapsed_times
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / self.elapsed_times.len() as f64;

        let total_time = self.start_time.elapsed().as_secs_f64();
        let overall_rate = self.total_updates as f64 / total_time;

        (1.0 / mean, variance, overall_rate)
    }

    fn log_to_file(&self, instant_rate: f64, variance: f64, overall_rate: f64) {
        if let Ok(mut file) = OpenOptions::new().append(true).open(&self.log_file) {
            let timestamp = self.start_time.elapsed().as_secs_f64();
            writeln!(
                file,
                "{},{},{},{},{}",
                timestamp, instant_rate, overall_rate, variance, self.total_updates
            )
            .unwrap_or_else(|e| eprintln!("Failed to write to log file: {}", e));
        }
    }
}

fn main() {
    let mut app = App::new();

    // Add minimal plugins without a window
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Time::<Fixed>::from_hz(1e6));

    // Add required plugins
    app.add_plugins((
        StartupSequencePlugin,
        PhysicsPlugin::with_config(PhysicsConfig {
            timestep: 1.0 / 1e6, // 1GHz physics timestep
            ..default()
        }),
    ));

    // Create aircraft config
    let aircraft_config = DubinsAircraftConfig {
        name: "test_aircraft".to_string(),
        max_speed: 200.0,
        min_speed: 40.0,
        acceleration: 10.0,
        max_bank_angle: 45.0 * std::f64::consts::PI / 180.0,
        max_turn_rate: 0.5,
        max_climb_rate: 5.0,
        max_descent_rate: 15.0,
        start_config: StartConfig::Fixed(flyer::components::FixedStartConfig {
            position: Vector3::new(0.0, 0.0, -500.0),
            speed: 100.0,
            heading: 0.0,
        }),
        task_config: Default::default(),
    };

    // Add aircraft plugin with config
    app.add_plugins(DubinsAircraftPlugin::new_single(aircraft_config));

    // Add physics update system
    app.add_systems(FixedUpdate, dubins_aircraft_system);

    // Add Metrics resource
    app.insert_resource(Metrics::default());
    app.add_systems(FixedUpdate, update_metrics);
    app.add_systems(FixedUpdate, log_metrics.run_if(every_n_seconds(1.0)));

    app.run();
}

// System to log aircraft state
fn every_n_seconds(seconds: f32) -> impl FnMut() -> bool {
    let mut last_run = Instant::now();
    move || {
        let now = Instant::now();
        if now.duration_since(last_run).as_secs_f32() >= seconds {
            last_run = now;
            true
        } else {
            false
        }
    }
}

fn update_metrics(mut metrics: ResMut<Metrics>) {
    metrics.update();
}

fn log_metrics(metrics: Res<Metrics>) {
    let (instant_rate, variance, overall_rate) = metrics.compute_stats();

    // Log to console
    println!(
        "Instant Rate: {} Hz, Overall Rate: {} Hz, Variance: {:.6e}, Total Updates: {}",
        instant_rate, overall_rate, variance, metrics.total_updates
    );

    // Log to file
    metrics.log_to_file(instant_rate, variance, overall_rate);
}
