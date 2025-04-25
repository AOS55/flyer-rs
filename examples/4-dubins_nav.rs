use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use flyer::{
    components::{DubinsAircraftConfig, DubinsAircraftState, StartConfig},
    plugins::{
        CameraPlugin, DubinsAircraftPlugin, PhysicsPlugin, StartupSequencePlugin, TerrainPlugin,
        TransformationPlugin,
    },
    resources::{PhysicsConfig, TerrainConfig},
    systems::{aircraft_render_system, dubins_aircraft_system},
};
use nalgebra::Vector3;

// TODO: Has some issues with the update and Sprite render is not there.

fn main() {
    let mut app = App::new();

    // Window setup
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Interactive Aircraft with Terrain".into(),
            resolution: (1024., 768.).into(),
            ..default()
        }),
        ..default()
    }));

    // Core plugins with terrain
    app.add_plugins((
        StartupSequencePlugin,
        TransformationPlugin::new(1.0),
        PhysicsPlugin::with_config(PhysicsConfig::default()),
        TerrainPlugin::with_config(TerrainConfig {
            seed: 42, // Change for different terrain
            ..default()
        }),
        CameraPlugin,
    ));

    // Aircraft config - start higher to see terrain better
    let aircraft_config = DubinsAircraftConfig {
        name: "interactive_aircraft".to_string(),
        max_speed: 200.0,
        min_speed: 40.0,
        acceleration: 10.0,
        max_bank_angle: 45.0 * std::f64::consts::PI / 180.0,
        max_turn_rate: 0.5,
        max_climb_rate: 5.0,
        max_descent_rate: 15.0,
        start_config: StartConfig::Fixed(flyer::components::FixedStartConfig {
            position: Vector3::new(0.0, 0.0, -1000.0), // Higher altitude
            speed: 100.0,
            heading: 0.0,
        }),
        task_config: Default::default(),
    };
    app.add_plugins(TilemapPlugin);
    // Add aircraft plugin
    app.add_plugins(DubinsAircraftPlugin::new_single(aircraft_config));

    // Add our systems
    app.add_systems(
        Update,
        (
            keyboard_control,
            dubins_aircraft_system,
            aircraft_render_system,
        )
            .chain(),
    );

    app.run();
}

fn keyboard_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut DubinsAircraftState>,
) {
    if let Ok(mut state) = query.get_single_mut() {
        let controls = &mut state.controls;
        // Acceleration control
        if keyboard.pressed(KeyCode::ArrowUp) {
            controls.acceleration = 10.0;
        } else if keyboard.pressed(KeyCode::ArrowDown) {
            controls.acceleration = -10.0;
        } else {
            controls.acceleration = 0.0;
        }

        // Bank angle control
        if keyboard.pressed(KeyCode::ArrowLeft) {
            controls.bank_angle -= 1.0 * time.delta_secs_f64();
        } else if keyboard.pressed(KeyCode::ArrowRight) {
            controls.bank_angle += 1.0 * time.delta_secs_f64();
        } else {
            controls.bank_angle *= 0.95; // Return to level
        }

        // Vertical speed control
        if keyboard.pressed(KeyCode::KeyW) {
            controls.vertical_speed += 5.0 * time.delta_secs_f64();
        } else if keyboard.pressed(KeyCode::KeyS) {
            controls.vertical_speed -= 5.0 * time.delta_secs_f64();
        } else {
            controls.vertical_speed *= 0.95; // Return to level
        }

        // Clamp controls
        controls.bank_angle = controls.bank_angle.clamp(-0.8, 0.8);
        controls.vertical_speed = controls.vertical_speed.clamp(-15.0, 5.0);
    }
}
