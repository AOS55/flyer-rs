use bevy::prelude::*;
use nalgebra::Vector3;

use crate::{
    components::{CameraComponent, PlayerController},
    plugins::StartupStage,
    resources::TransformationResource,
    systems::camera_follow_system,
};

/// Plugin to manage the camera setup and behavior in the game.
/// - Spawns the camera at startup.
/// - Handles camera-follow behavior in fixed updates.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    /// Builds the `CameraPlugin` by registering the required systems.
    fn build(&self, app: &mut App) {
        app.add_systems(
            // Run the camera spawn system during the startup phase
            Startup,
            spawn_camera.in_set(StartupStage::BuildCameras),
        )
        .add_systems(
            // Run the camera-follow system during fixed updates for consistent movement
            FixedUpdate,
            camera_follow_system,
        );
    }
}

/// System to spawn the initial camera and attach it to the player entity.
///
/// # Arguments:
/// - `commands`: Used to spawn the camera entity.
/// - `player_query`: Query to find the player entity with the `PlayerController` component.
/// - `transform_res`: Resource for handling coordinate transformations between physics space and render space.
fn spawn_camera(
    mut commands: Commands,
    player_query: Query<Entity, With<PlayerController>>,
    transform_res: Res<TransformationResource>,
) {
    // Attempt to get the single player entity from the query
    // TODO: Handle the case where there are multiple player entities
    if let Ok(player_entity) = player_query.get_single() {
        // 1. Set the initial camera position in the NED (North-East-Down) coordinate system
        let initial_pos = Vector3::new(0.0, 0.0, 500.0);

        // 2. Convert the position to screen/render space using the transformation resource
        let render_pos = transform_res.screen_from_ned(&initial_pos).unwrap();

        // 3. Spawn the camera entity with core camera components and a custom `CameraComponent`
        commands.spawn((
            // Core camera components for 2D
            Camera2d::default(), // Basic 2D camera
            Camera {
                hdr: true, // Enable HDR rendering for better visual quality
                ..default()
            },
            Transform::from_translation(render_pos), // Set the camera position in world space
            GlobalTransform::default(),              // Initialize the global transform
            // The custom camera component
            CameraComponent {
                target: Some(player_entity), // Set the player entity as the target for the camera
                ..default()
            },
        ));
    } else {
        warn!("No player entity found when spawning camera!");
    }
}
