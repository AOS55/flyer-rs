// src/systems/camera/controller.rs

use bevy::prelude::*;

use crate::{
    components::{
        DubinsAircraftState, // Needed for Dubins model
        PlayerController,
        SpatialComponent, // Needed for Full model
    },
    resources::{Frame, PositionTransform, TransformationResource},
};

/// System for making the camera follow a target aircraft (Dubins or Full).
pub fn camera_follow_system(
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    // Query for either SpatialComponent OR DubinsAircraftState on the player entity
    target_query: Query<
        (
            Option<&SpatialComponent>,    // Direct SpatialComponent (Full model)
            Option<&DubinsAircraftState>, // DubinsAircraftState (contains SpatialComponent)
        ),
        With<PlayerController>, // Filter for the player-controlled entity
    >,
    transform_res: Res<TransformationResource>,
) {
    if let Ok((mut camera_transform, mut projection)) = camera_query.get_single_mut() {
        // Iterate through potential targets (should only be one with PlayerController)
        for (spatial_direct_opt, dubins_state_opt) in target_query.iter() {
            // --- Try to get the SpatialComponent ---
            let target_spatial: Option<&SpatialComponent> = if let Some(spatial) =
                spatial_direct_opt
            {
                // Found direct SpatialComponent (Full Aircraft)
                Some(spatial)
            } else if let Some(dubins_state) = dubins_state_opt {
                // Found DubinsAircraftState, get nested spatial component
                Some(&dubins_state.spatial)
            } else {
                // Entity has PlayerController but neither expected state component? Log warning.
                warn!("PlayerController entity found, but lacks SpatialComponent or DubinsAircraftState!");
                None
            };
            // --- End SpatialComponent extraction ---

            // If we successfully got a SpatialComponent (either direct or nested)
            if let Some(spatial) = target_spatial {
                let ned_pos = spatial.position; // Use the extracted position

                // Convert the NED position to screen/render coordinates
                match transform_res.transform_to_screen_coords(&ned_pos, Frame::NED) {
                    Ok(screen_pos) => {
                        // Update camera position
                        camera_transform.translation.x = screen_pos.x;
                        camera_transform.translation.y = screen_pos.y;

                        // Update camera scale (zoom) based on altitude
                        let altitude = -ned_pos.z;
                        let base_scale = 1.0;
                        let calculated_scale = (base_scale * (1.0 + altitude / 1000.0)).max(0.1);
                        projection.scale = calculated_scale as f32;
                    }
                    Err(e) => {
                        error!("Failed to transform target position for camera: {}", e);
                    }
                }
                // Since we found and processed the player, break the loop (there should only be one)
                break;
            }
        }
    }
}
