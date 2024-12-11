use bevy::prelude::*;

use crate::components::{DubinsAircraftState, PlayerController};
use crate::resources::{Frame, PositionTransform, TransformationResource};

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

        // Set camera position to match aircraft, keeping original z coordinate
        if let Ok(screen_pos) = transform_res.transform_to_screen_coords(&ned_pos, Frame::NED) {
            // Update camera position to match aircraft position,
            camera_transform.translation = screen_pos;

            let altitude = -ned_pos.z;
            let base_scale = 1.0;
            projection.scale = (base_scale * (1.0 + altitude / 1000.0)) as f32;
        }
    }
}
