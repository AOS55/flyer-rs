use bevy::prelude::*;
use nalgebra::Vector3;

use crate::components::{CameraComponent, DubinsAircraftState, PlayerController};
use crate::resources::{Frame, PositionTransform, TransformationResource};

pub fn camera_follow_system(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    target_query: Query<&DubinsAircraftState, With<PlayerController>>,
    transform_res: Res<TransformationResource>,
    time: Res<Time>,
) {
    // Get the camera and target positions
    if let (Ok(mut camera_transform), Ok(state)) =
        (camera_query.get_single_mut(), target_query.get_single())
    {
        // Convert aircraft position to render coordinates
        let ned_pos = state.spatial.position;

        // Set camera position to match aircraft, keeping original z coordinate
        if let Ok(screen_pos) = transform_res.transform_to_screen_coords(&ned_pos, Frame::NED) {
            // Update camera position to match aircraft position, keeping z-coordinate
            camera_transform.translation = Vec3::new(
                screen_pos.x,
                screen_pos.y,
                camera_transform.translation.z, // Maintain camera z position
            );
        }
    }
}
