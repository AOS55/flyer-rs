use bevy::prelude::*;

use crate::components::{DubinsAircraftState, PlayerController};
use crate::resources::{Frame, PositionTransform, TransformationResource};

/// System for making the camera follow a target aircraft.
///
/// This system updates the camera's position and scale to follow the aircraft controlled
/// by the player. The camera's position is transformed from the aircraft's NED (North-East-Down)
/// coordinates to screen/render coordinates. The camera scale adjusts dynamically based on
/// the aircraft's altitude.
pub fn camera_follow_system(
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    target_query: Query<&DubinsAircraftState, With<PlayerController>>,
    transform_res: Res<TransformationResource>,
) {
    // Get the camera and target positions
    if let (Ok((mut camera_transform, mut projection)), Ok(state)) =
        (camera_query.get_single_mut(), target_query.get_single())
    {
        // Convert aircraft position to render coordinates
        let ned_pos = state.spatial.position;

        // Convert the NED position to screen/render coordinates
        if let Ok(screen_pos) = transform_res.transform_to_screen_coords(&ned_pos, Frame::NED) {
            // Update the camera's position, keeping the original Z coordinate for 2D projection
            camera_transform.translation = screen_pos;

            // Adjust the camera's scale dynamically based on the aircraft's altitude
            let altitude = -ned_pos.z;
            let base_scale = 1.0;
            projection.scale = (base_scale * (1.0 + altitude / 1000.0)) as f32;
        }
    }
}
