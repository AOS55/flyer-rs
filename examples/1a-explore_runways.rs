use bevy::prelude::*;
use flyer::{
    components::RunwayComponent,
    plugins::{RunwayPlugin, StartupSequencePlugin, TransformationPlugin},
    systems::spawn_runway_sprite,
};
use nalgebra::Vector3;
use std::f64::consts::PI;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Runway Viewer".into(),
            resolution: (800., 600.).into(),
            ..default()
        }),
        ..default()
    }));

    // --- Define your desired Runway Configuration ---
    let runway_config_data = RunwayComponent {
        position: Vector3::zeros(),    // Example: At origin
        heading: 120.0 * (PI / 180.0), // Example: 120 degrees
        width: 45.0,                   // Example: 45 meters wide
        length: 1500.0,                // Example: 1500 meters long
    };
    // --- End Configuration ---

    app.add_plugins((
        StartupSequencePlugin,
        TransformationPlugin::new(1.0),
        RunwayPlugin::new(Some(runway_config_data)),
    ));

    app.add_systems(Startup, setup_camera);
    app.add_systems(Update, pan_camera); // Keep camera controls
                                         // app.add_systems(Update, spawn_runway_sprite);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 999.9)),
        GlobalTransform::default(),
    ));
}

fn pan_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    let (mut transform, mut projection) = query.single_mut();
    let mut direction = Vec3::ZERO;
    let speed = 1600.0;

    if keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    if direction != Vec3::ZERO {
        transform.translation +=
            direction.normalize() * speed * time.delta_secs() * projection.scale;
    }

    let min_scale = 0.1;
    let max_scale = 10.0;

    if keyboard.pressed(KeyCode::Equal) {
        projection.scale = (projection.scale * 0.95).max(min_scale);
    }
    if keyboard.pressed(KeyCode::Minus) {
        projection.scale = (projection.scale * 1.05).min(max_scale);
    }
}
