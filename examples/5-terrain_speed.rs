use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use flyer::plugins::{StartupSequencePlugin, TerrainPlugin, TransformationPlugin};
use std::time::Duration;
use std::{fs::OpenOptions, io::Write, path::PathBuf};

// Resource to store performance metrics
#[derive(Resource)]
pub struct PerformanceMetrics {
    frame_times: Vec<Vec<Duration>>, // Separate vec for each test configuration
    current_zoom_index: usize,
    current_test_type: TestType,
    samples_collected: usize,
    test_complete: bool,
    log_file: PathBuf,
}

#[derive(PartialEq, Clone, Copy)]
enum TestType {
    Static,
    Moving,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        let log_file = std::env::temp_dir().join("terrain_performance_metrics.csv");

        // Create/overwrite the file with headers
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&log_file)
        {
            writeln!(
                file,
                "test_type,zoom_level,avg_fps,avg_frame_time_ms,min_frame_time_ms,max_frame_time_ms,variance"
            )
            .unwrap();
        }

        println!("Log file created at: {:?}", log_file);

        // Initialize vectors for each test configuration (static and moving, for each zoom level)
        let total_configurations = ZOOM_LEVELS.len() * 2; // 2 for static and moving
        let frame_times = vec![Vec::with_capacity(SAMPLES_PER_ZOOM); total_configurations];

        Self {
            frame_times,
            current_zoom_index: 0,
            current_test_type: TestType::Static,
            samples_collected: 0,
            test_complete: false,
            log_file,
        }
    }
}

// Component to mark our test camera
#[derive(Component)]
pub struct TestCamera;

// Plugin to handle performance testing
pub struct PerformanceTestPlugin;

impl Plugin for PerformanceTestPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerformanceMetrics>()
            .add_systems(Startup, setup_performance_test)
            .add_systems(Update, (collect_metrics, move_camera));
    }
}

const ZOOM_LEVELS: [f32; 3] = [0.1, 1.0, 5.0]; // Min, Medium, Max zoom
const SAMPLES_PER_ZOOM: usize = 10; // Number of frames to collect per zoom level
const CAMERA_SPEED: f32 = 500.0; // Units per second

fn setup_performance_test(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 999.9)),
        GlobalTransform::default(),
        TestCamera,
    ));
}

fn move_camera(
    time: Res<Time>,
    metrics: Res<PerformanceMetrics>,
    mut query: Query<&mut Transform, With<TestCamera>>,
) {
    if metrics.current_test_type == TestType::Moving {
        if let Ok(mut transform) = query.get_single_mut() {
            transform.translation.x += CAMERA_SPEED * time.delta_secs();
        }
    }
}

fn collect_metrics(
    time: Res<Time>,
    mut metrics: ResMut<PerformanceMetrics>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<TestCamera>>,
) {
    if metrics.test_complete {
        return;
    }

    // Calculate current test configuration index
    let config_index = metrics.current_zoom_index
        + (if metrics.current_test_type == TestType::Moving {
            ZOOM_LEVELS.len()
        } else {
            0
        });

    // Collect frame time
    metrics.frame_times[config_index].push(time.delta());
    metrics.samples_collected += 1;

    // Print progress
    if metrics.samples_collected % 10 == 0 {
        println!(
            "Progress: {}/{} samples for test at zoom level {}",
            metrics.frame_times[config_index].len(),
            SAMPLES_PER_ZOOM,
            ZOOM_LEVELS[metrics.current_zoom_index]
        );
    }

    // Check if we need to move to the next configuration
    if metrics.frame_times[config_index].len() >= SAMPLES_PER_ZOOM {
        // Move to next zoom level or test type
        if metrics.current_zoom_index + 1 < ZOOM_LEVELS.len() {
            // Move to next zoom level
            metrics.current_zoom_index += 1;
        } else if metrics.current_test_type == TestType::Static {
            // Switch to moving tests
            metrics.current_test_type = TestType::Moving;
            metrics.current_zoom_index = 0;
            // Reset camera position
            if let Ok((mut transform, _)) = query.get_single_mut() {
                transform.translation = Vec3::new(0.0, 0.0, 999.9);
            }
        } else {
            // All tests complete
            metrics.test_complete = true;
            output_results(&metrics);
            return;
        }

        // Update projection for new zoom level
        if let Ok((_, mut projection)) = query.get_single_mut() {
            projection.scale = ZOOM_LEVELS[metrics.current_zoom_index];
        }
    }
}

fn output_results(metrics: &PerformanceMetrics) {
    if let Ok(mut file) = OpenOptions::new().append(true).open(&metrics.log_file) {
        for test_type in [TestType::Static, TestType::Moving] {
            for (zoom_index, &zoom_level) in ZOOM_LEVELS.iter().enumerate() {
                let config_index = zoom_index
                    + (if test_type == TestType::Moving {
                        ZOOM_LEVELS.len()
                    } else {
                        0
                    });

                let test_name = match test_type {
                    TestType::Static => "static",
                    TestType::Moving => "moving",
                };

                let samples = &metrics.frame_times[config_index];

                // Calculate statistics
                let avg_frame_time = samples.iter().sum::<Duration>() / samples.len() as u32;
                let avg_fps = 1.0 / avg_frame_time.as_secs_f64();
                let min_frame_time = samples.iter().min().unwrap();
                let max_frame_time = samples.iter().max().unwrap();

                // Calculate variance
                let mean_secs = avg_frame_time.as_secs_f64();
                let variance = samples
                    .iter()
                    .map(|&x| {
                        let diff = x.as_secs_f64() - mean_secs;
                        diff * diff
                    })
                    .sum::<f64>()
                    / samples.len() as f64;

                // Write to CSV
                writeln!(
                    file,
                    "{},{},{},{},{},{},{}",
                    test_name,
                    zoom_level,
                    avg_fps,
                    avg_frame_time.as_secs_f64() * 1000.0,
                    min_frame_time.as_secs_f64() * 1000.0,
                    max_frame_time.as_secs_f64() * 1000.0,
                    variance
                )
                .unwrap_or_else(|e| eprintln!("Failed to write to log file: {}", e));

                // Also print to console
                println!("\nTest Type: {}, Zoom Level: {}", test_name, zoom_level);
                println!("Average FPS: {:.2}", avg_fps);
                println!("Average Frame Time: {:.2?}", avg_frame_time);
                println!("Min Frame Time: {:.2?}", min_frame_time);
                println!("Max Frame Time: {:.2?}", max_frame_time);
                println!("Variance: {:.6e}", variance);
            }
        }
    }
}

// Add this to your main.rs:
fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Terrain Viewer".into(),
            resolution: (800., 600.).into(),
            ..default()
        }),
        ..default()
    }));

    app.add_plugins((
        StartupSequencePlugin,
        TransformationPlugin::new(1.0),
        TerrainPlugin::new(),
    ));

    app.add_plugins(TilemapPlugin);

    // Add the performance test plugin
    app.add_plugins(PerformanceTestPlugin);

    app.run();
}
