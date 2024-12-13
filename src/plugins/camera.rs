use bevy::prelude::*;
use nalgebra::Vector3;

use crate::components::{CameraComponent, PlayerController};
use crate::plugins::StartupSet;
use crate::resources::TransformationResource;
use crate::systems::camera_follow_system;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera.in_set(StartupSet::SpawnCamera))
            .add_systems(FixedUpdate, camera_follow_system);
    }
}

fn spawn_camera(
    mut commands: Commands,
    player_query: Query<Entity, With<PlayerController>>,
    transform_res: Res<TransformationResource>,
) {
    if let Ok(player_entity) = player_query.get_single() {
        // Initial camera position in physics space
        let initial_pos = Vector3::new(0.0, 0.0, 500.0);

        // Convert to render space
        let render_pos = transform_res.screen_from_ned(&initial_pos).unwrap();

        commands.spawn((
            // Core camera components for 2D
            Camera2d::default(),
            Camera {
                hdr: true,
                ..default()
            },
            Transform::from_translation(render_pos),
            GlobalTransform::default(),
            // Our custom camera component
            CameraComponent {
                target: Some(player_entity),
                ..default()
            },
        ));
    } else {
        warn!("No player entity found when spawning camera!");
    }
}
