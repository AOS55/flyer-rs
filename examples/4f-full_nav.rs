use bevy::{
    prelude::*,
    // pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
};
use flyer::{
    components::{
        AircraftConfig,
        AircraftControlSurfaces,
        AircraftRenderState, // Needed for aircraft_render_system logging
        CameraComponent, // Needed for camera_follow_system logging (if CameraPlugin doesn't expose it)
        FullAircraftConfig,
        PlayerController,
        // Import components needed for logging systems if they aren't public
        SpatialComponent, // Needed for physics_integrator_system logging
        StartConfig,
    },
    plugins::{
        add_aircraft_plugin, CameraPlugin,
        /* ComplexPhysicsSet, */ EnvironmentPlugin, /* FullAircraftPlugin, */
        PhysicsPlugin, RunwayPlugin, StartupSequencePlugin, TerrainPlugin, TransformationPlugin,
        TrimPlugin,
    },
    resources::{
        AircraftAssets, PhysicsConfig, TransformationResource, /* Keep if used directly */
    }, // Adjusted resources
    systems::{
        aero_force_system, air_data_system,
        aircraft_render_system, /* camera_follow_system is added by CameraPlugin */
        collision_detection_system, force_calculator_system, handle_trim_requests,
        physics_integrator_system, propulsion_system, spawn_aircraft_sprite, trim_aircraft_system,
    },
};
use nalgebra::Vector3;
use std::f64::consts::PI;

// Constants remain the same
const MAX_DEFLECTION_RATE: f64 = PI / 2.0;
const THROTTLE_RATE: f64 = 0.5;

fn main() {
    let mut app = App::new();

    // --- Core Plugins ---
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Full Aircraft Keyboard Control".into(),
            resolution: (1024., 768.).into(),
            ..default()
        }),
        ..default()
    }));

    // --- Flyer Plugins ---
    app.add_plugins((
        StartupSequencePlugin,
        TransformationPlugin::new(1.0),
        EnvironmentPlugin::new(),
        PhysicsPlugin::with_config(PhysicsConfig::default()),
        TerrainPlugin::new(),
        CameraPlugin, // This adds camera_follow_system implicitly
        RunwayPlugin::new(None),
        TrimPlugin,
    ));

    // --- Aircraft Setup ---
    let mut twin_otter_config = FullAircraftConfig::twin_otter();
    twin_otter_config.start_config = StartConfig::Fixed(flyer::components::FixedStartConfig {
        position: Vector3::new(0.0, 0.0, -1500.0),
        speed: 70.0,
        heading: 0.0,
    });
    add_aircraft_plugin(&mut app, AircraftConfig::Full(twin_otter_config));

    // --- Systems ---
    app.add_systems(Update, full_aircraft_keyboard_control);
    // app.add_systems(
    //     PostUpdate,
    //     // spawn_aircraft_sprite.run_if(run_if_resource_exists::<AircraftAssets>()),
    // );

    // Add physics, trim, rendering, and LOGGING systems to FixedUpdate
    app.add_systems(
        FixedUpdate,
        (
            // Physics Calculations
            (
                air_data_system,
                aero_force_system,
                propulsion_system,
                force_calculator_system,
            )
                .chain(),
            // Integration & Logging
            (
                physics_integrator_system, // Integrator runs first
                log_physics_state,         // Log state *after* integration
            )
                .chain(),
            // Rendering Updates & Logging
            (
                aircraft_render_system, // Update sprite transform
                log_render_state,       // Log the render state
            )
                .chain(),
            // Camera Logging (Camera follow system is added by plugin)
            log_camera_state,
        )
            .chain(),
    );

    app.run();
}

// --- Keyboard Control System (Simplified Auto-Centering) ---
fn full_aircraft_keyboard_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut AircraftControlSurfaces, With<PlayerController>>,
) {
    if let Ok(mut controls) = query.get_single_mut() {
        let dt = time.delta_secs_f64();
        let mut input_detected = false;

        // --- Elevator ---
        let mut elevator_change = 0.0;
        if keyboard.pressed(KeyCode::ArrowDown) {
            elevator_change -= MAX_DEFLECTION_RATE * dt;
            input_detected = true;
        }
        if keyboard.pressed(KeyCode::ArrowUp) {
            elevator_change += MAX_DEFLECTION_RATE * dt;
            input_detected = true;
        }
        controls.elevator += elevator_change;

        // --- Aileron ---
        let mut aileron_change = 0.0;
        if keyboard.pressed(KeyCode::ArrowLeft) {
            aileron_change += MAX_DEFLECTION_RATE * dt;
            input_detected = true;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            aileron_change -= MAX_DEFLECTION_RATE * dt;
            input_detected = true;
        }
        controls.aileron += aileron_change;

        // --- Rudder ---
        let mut rudder_change = 0.0;
        if keyboard.pressed(KeyCode::KeyA) {
            rudder_change += MAX_DEFLECTION_RATE * dt;
            input_detected = true;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            rudder_change -= MAX_DEFLECTION_RATE * dt;
            input_detected = true;
        }
        controls.rudder += rudder_change;

        // --- Throttle ---
        let mut throttle_change = 0.0;
        if keyboard.pressed(KeyCode::KeyW) {
            throttle_change += THROTTLE_RATE * dt;
            input_detected = true;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            throttle_change -= THROTTLE_RATE * dt;
            input_detected = true;
        }
        controls.power_lever += throttle_change;

        // --- Clamping ---
        controls.elevator = controls.elevator.clamp(-1.0, 1.0);
        controls.aileron = controls.aileron.clamp(-1.0, 1.0);
        controls.rudder = controls.rudder.clamp(-1.0, 1.0);
        controls.power_lever = controls.power_lever.clamp(0.0, 1.0);

        // --- Simplified Auto-Centering: Remove for now ---
        // let centering_rate = 2.0;
        // let damping_factor = (-centering_rate * dt).exp();
        // if elevator_change == 0.0 { controls.elevator *= damping_factor; }
        // if aileron_change == 0.0 { controls.aileron *= damping_factor; }
        // if rudder_change == 0.0 { controls.rudder *= damping_factor; }
        // if controls.elevator.abs() < 1e-4 { controls.elevator = 0.0; }
        // if controls.aileron.abs() < 1e-4 { controls.aileron = 0.0; }
        // if controls.rudder.abs() < 1e-4 { controls.rudder = 0.0; }

        // --- Logging ---
        if input_detected {
            info!(
                "[Input] Controls Set: E={:.2} A={:.2} R={:.2} P={:.2}",
                controls.elevator, controls.aileron, controls.rudder, controls.power_lever
            );
        }
    }
}

// --- Logging Systems ---

fn log_physics_state(query: Query<&SpatialComponent, With<PlayerController>>) {
    if let Ok(spatial) = query.get_single() {
        info!(
            "[Physics] Spatial State Updated: Pos=({:.1}, {:.1}, {:.1}) Vel=({:.1}, {:.1}, {:.1})",
            spatial.position.x,
            spatial.position.y,
            spatial.position.z,
            spatial.velocity.x,
            spatial.velocity.y,
            spatial.velocity.z
        );
    }
}

fn log_render_state(
    query: Query<
        (&Transform, &ViewVisibility),
        (With<AircraftRenderState>, With<PlayerController>),
    >,
) {
    if let Ok((transform, visibility)) = query.get_single() {
        if visibility.get() {
            // Check if entity is potentially visible
            info!(
                "[Render] Aircraft Transform Updated: Translation=({:.1}, {:.1}, {:.1})",
                transform.translation.x, transform.translation.y, transform.translation.z
            );
        } else {
            info!("[Render] Aircraft Render State exists but entity is not visible.");
        }
    } else {
        // This might spam logs if entity exists but lacks Transform/Visibility yet
        // info!("[Render] Query for Aircraft Render State failed this frame.");
    }
}

fn log_camera_state(
    cam_query: Query<&Transform, With<Camera2d>>,
    target_query: Query<Entity, With<PlayerController>>,
) {
    if let Ok(cam_transform) = cam_query.get_single() {
        let target_exists = target_query.get_single().is_ok();
        info!(
            "[Camera] Camera State: Pos=({:.1}, {:.1}, {:.1}), Player Target Exists={}",
            cam_transform.translation.x,
            cam_transform.translation.y,
            cam_transform.translation.z,
            target_exists
        );
    } else {
        info!("[Camera] Query for Camera2d failed this frame.");
    }
}
