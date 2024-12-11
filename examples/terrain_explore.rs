use bevy::prelude::*;
use flyer::plugins::terrain::TerrainPlugin;
use flyer::resources::terrain::{RenderConfig, TerrainConfig, TerrainState};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TerrainPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, pan_camera)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    // Initialize terrain configuration
    let terrain_config = TerrainConfig {
        noise: Default::default(),
        biome: Default::default(),
        feature: Default::default(),
        render: RenderConfig {
            feature_layer_offset: 10.0,
        },
    };

    // Initialize terrain state
    let terrain_state = TerrainState {
        // Core parameters
        chunk_size: 32,
        scale: 1.0,
        seed: 42,

        // Runtime state
        active_chunks: Vec::new(),
        tile_size: 16.0,
        chunks_to_load: Default::default(),
        chunks_to_unload: Default::default(),

        // Loading parameters
        loading_radius: 10,
        max_chunks_per_frame: 8,
    };

    // Insert resources
    commands.insert_resource(terrain_config);
    commands.insert_resource(terrain_state);

    // Spawn camera
    commands.spawn((Camera2d::default(), MainCamera));
}

fn pan_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
) {
    let (mut transform, mut projection) = query.single_mut();
    let mut direction = Vec3::ZERO;
    let speed = 400.0;

    // info!(
    //     "Camera position: {}, Scale: {}",
    //     transform.translation, projection.scale
    // );

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
