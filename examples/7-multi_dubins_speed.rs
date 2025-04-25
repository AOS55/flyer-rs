use bevy::prelude::*;
use chrono;
use flyer::{
    components::{DubinsAircraftConfig, StartConfig},
    plugins::{
        AircraftBaseInitialized, DubinsAircraftPlugin, PhysicsPlugin, StartupSequencePlugin,
    },
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
    aircraft_count: usize,
    should_exit: bool,
    test_duration: std::time::Duration,
}

impl Default for Metrics {
    fn default() -> Self {
        let log_file = std::env::temp_dir().join("multi_aircraft_metrics.csv");
        // Create/overwrite the file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file)
        {
            writeln!(
                file,
                "timestamp,aircraft_count,instant_rate,overall_rate,variance,total_updates"
            )
            .unwrap();
        }

        println!("Log file created at: {:?}", log_file);

        Self {
            elapsed_times: Vec::with_capacity(1000),
            last_update: Instant::now(),
            start_time: Instant::now(),
            total_updates: 0,
            log_file,
            aircraft_count: 0,
            should_exit: false,
            test_duration: std::time::Duration::from_secs(30), // Default test duration
        }
    }
}

impl Metrics {
    fn new(aircraft_count: usize, test_duration: std::time::Duration, log_dir: PathBuf) -> Self {
        let log_file = log_dir.join(format!("run_{:04}_aircraft.csv", aircraft_count));

        // Create/overwrite the file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file)
        {
            writeln!(
                file,
                "timestamp,aircraft_count,instant_rate,overall_rate,variance,total_updates"
            )
            .unwrap();
        }

        println!("Log file created at: {:?}", log_file);

        Self {
            elapsed_times: Vec::with_capacity(1000),
            last_update: Instant::now(),
            start_time: Instant::now(),
            total_updates: 0,
            log_file,
            aircraft_count,
            should_exit: false,
            test_duration,
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

        // Check if we should exit
        if now.duration_since(self.start_time) >= self.test_duration {
            self.should_exit = true;
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
                "{},{},{},{},{},{}",
                timestamp,
                self.aircraft_count,
                instant_rate,
                overall_rate,
                variance,
                self.total_updates
            )
            .unwrap_or_else(|e| eprintln!("Failed to write to log file: {}", e));
        }
    }
}

fn create_aircraft_config(index: usize) -> DubinsAircraftConfig {
    // Create different starting positions for each aircraft
    let x_offset = (index as f64 % 10.0) * 100.0;
    let y_offset = (index as f64 / 10.0).floor() * 100.0;

    DubinsAircraftConfig {
        name: format!("aircraft_{}", index),
        max_speed: 200.0,
        min_speed: 40.0,
        acceleration: 10.0,
        max_bank_angle: 45.0 * std::f64::consts::PI / 180.0,
        max_turn_rate: 0.5,
        max_climb_rate: 5.0,
        max_descent_rate: 15.0,
        start_config: StartConfig::Fixed(flyer::components::FixedStartConfig {
            position: Vector3::new(x_offset, y_offset, -500.0),
            speed: 100.0,
            heading: 0.0,
        }),
        task_config: Default::default(),
    }
}

fn run_simulation(n_aircraft: usize, test_duration: std::time::Duration, log_dir: PathBuf) {
    println!("Starting simulation with {} aircraft", n_aircraft);

    let mut app = App::new();

    // Add minimal plugins without a window
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Time::<Fixed>::from_hz(1e6));

    // Create configurations for all aircraft
    let configs = (0..n_aircraft).map(|i| create_aircraft_config(i)).collect();
    app.insert_resource(AircraftBaseInitialized);

    // Add required plugins
    app.add_plugins((
        StartupSequencePlugin,
        PhysicsPlugin::with_config(PhysicsConfig {
            timestep: 1.0 / 1e6,
            ..default()
        }),
        DubinsAircraftPlugin::new_vector(configs),
    ));

    // Add physics update system
    app.add_systems(FixedUpdate, dubins_aircraft_system);

    // Add Metrics resource with new constructor
    app.insert_resource(Metrics::new(n_aircraft, test_duration, log_dir));

    app.add_systems(FixedUpdate, update_metrics);
    app.add_systems(FixedUpdate, log_metrics.run_if(every_n_seconds(1.0)));
    app.add_systems(Update, check_exit);

    app.run();
}

fn main() {
    let test_duration = std::time::Duration::from_secs(20); // 20 seconds per test
    let aircraft_counts = [1, 10, 100, 1000, 10000, 100000]; // Array of aircraft counts to test

    // Create directory and summary file once at the start
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let log_dir = std::env::temp_dir().join(format!("aircraft_metrics_{}", timestamp));
    std::fs::create_dir_all(&log_dir).unwrap();

    let summary_file = log_dir.join("summary.csv");

    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&summary_file)
    {
        writeln!(
            file,
            "aircraft_count,average_rate,final_variance,test_duration_seconds"
        )
        .unwrap();
    }

    println!("Created output directory at: {:?}", log_dir);

    for &n_aircraft in aircraft_counts.iter() {
        run_simulation(n_aircraft, test_duration, log_dir.clone());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    println!("All tests completed. Results are in: {:?}", log_dir);
}

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
        "Aircraft: {}, Instant Rate: {} Hz, Overall Rate: {} Hz, Variance: {:.6e}, Total Updates: {}",
        metrics.aircraft_count, instant_rate, overall_rate, variance, metrics.total_updates
    );

    // Log to file
    metrics.log_to_file(instant_rate, variance, overall_rate);
}

fn check_exit(metrics: Res<Metrics>, mut exit: EventWriter<bevy::app::AppExit>) {
    if metrics.should_exit {
        exit.send(bevy::app::AppExit::default());
    }
}
