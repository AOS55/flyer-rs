use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use flyer::plugins::{StartupSequencePlugin, TerrainPlugin, TransformationPlugin};

fn main() {
    let mut app = App::new();

    // Add default plugins with a window
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Terrain Viewer".into(),
            resolution: (800., 600.).into(),
            ..default()
        }),
        ..default()
    }));

    // Add our required plugins
    app.add_plugins((
        StartupSequencePlugin,
        TransformationPlugin::new(1.0),
        TerrainPlugin::new(),
    ));
    app.add_plugins(TilemapPlugin);

    // Add camera
    app.add_systems(Startup, setup_camera);
    // Add keyboard movement system
    app.add_systems(Update, pan_camera);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        // Core camera components for 2D
        Camera2d::default(), // Basic 2D camera
        Camera {
            hdr: true, // Enable HDR rendering for better visual quality
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 999.9)), // Set the camera position in world space
        GlobalTransform::default(),                              // Initialize the global transform
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

    // Zoom controls with limits
    let min_scale = 0.1;
    let max_scale = 10.0;

    if keyboard.pressed(KeyCode::Equal) {
        projection.scale = (projection.scale * 0.95).max(min_scale);
    }
    if keyboard.pressed(KeyCode::Minus) {
        projection.scale = (projection.scale * 1.05).min(max_scale);
    }
}
