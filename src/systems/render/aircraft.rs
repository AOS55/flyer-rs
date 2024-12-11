use bevy::prelude::*;

use crate::components::{AircraftRenderState, AircraftType, Attitude, DubinsAircraftState};
use crate::resources::{AircraftAssets, Frame, PositionTransform, TransformationResource};

pub fn aircraft_render_system(
    mut query: Query<(
        &DubinsAircraftState,
        &mut AircraftRenderState,
        &mut Sprite,
        &mut Transform,
    )>,
    transform_res: Res<TransformationResource>,
) {
    for (state, mut render_state, mut sprite, mut transform) in query.iter_mut() {
        // Handle sprite attitude updates
        let (roll, pitch, yaw) = state.spatial.attitude.euler_angles();
        let new_attitude = Attitude::from_angles(pitch, roll);
        if render_state.attitude != new_attitude {
            render_state.attitude = new_attitude;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = new_attitude.to_index();
            }
        }

        let ned_pos = &state.spatial.position;
        info!("NED position: {:?}", ned_pos);
        if let Ok(screen_pos) = transform_res.transform_to_screen_coords(ned_pos, Frame::NED) {
            // Keep z-coordinate from original transform for proper layering
            transform.translation = Vec3::new(screen_pos.x, screen_pos.y, screen_pos.z);
            info!("screen pos: {}", screen_pos);
        }
        transform.rotation = Quat::from_rotation_z(-yaw as f32);
    }
}

pub fn spawn_aircraft_sprite(
    mut commands: Commands,
    query: Query<(Entity, &AircraftType), (With<AircraftRenderState>, Without<Sprite>)>,
    aircraft_assets: Res<AircraftAssets>,
) {
    info!("Attempting to spawn aircraft sprites...");
    for (entity, ac_type) in query.iter() {
        info!(
            "Found aircraft entity: {:?} with type: {:?}",
            entity, ac_type
        );
        if let (Some(texture), Some(layout)) = (
            aircraft_assets.aircraft_textures.get(&ac_type),
            aircraft_assets.aircraft_layouts.get(&ac_type),
        ) {
            commands.entity(entity).insert((
                Sprite::from_atlas_image(
                    texture.clone(),
                    TextureAtlas {
                        layout: layout.clone(),
                        index: Attitude::Level.to_index(),
                    },
                ),
                Transform::from_xyz(0.0, 0.0, 10.0),
            ));
        } else {
            warn!("Missing texture or layout for aircraft type: {:?}", ac_type);
        }
    }
}
