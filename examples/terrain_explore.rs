use bevy::prelude::*;
use flyer::plugins::terrain::{TerrainPlugin, TerrainPluginConfig};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TerrainPlugin::new(TerrainPluginConfig {
                world_size: IVec2::new(1000, 1000),
                chunk_size: 32,
                seed: 42,
                scale: 16.0, // Adjusted to match tile scale
                max_concurrent_chunks: 10,
            }),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, pan_camera)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    // Spawn the main camera
    commands.spawn((Camera2d::default(), MainCamera));
}

fn pan_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
) {
    let (mut transform, mut projection) = query.single_mut();
    let mut direction = Vec3::ZERO;
    let speed = 200.0;

    info!(
        "Camera position: {}, Scale: {}",
        transform.translation, projection.scale
    );

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
        projection.scale = (projection.scale * 0.99).max(min_scale);
    }
    if keyboard.pressed(KeyCode::Minus) {
        projection.scale = (projection.scale * 1.01).min(max_scale);
    }
}
