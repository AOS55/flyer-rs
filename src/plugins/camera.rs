use crate::components::FlightCamera;
use crate::systems::camera::*;
use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CameraSet {
    Follow,
    Zoom,
}

pub struct FlightCameraPlugin;

impl Plugin for FlightCameraPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register the component
            .register_type::<FlightCamera>()
            // Configure system sets
            .configure_sets(Update, (CameraSet::Follow, CameraSet::Zoom).chain())
            // Add systems
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    camera_follow_system.in_set(CameraSet::Follow),
                    camera_zoom_system.in_set(CameraSet::Zoom),
                    update_camera_target,
                ),
            );
    }
}
