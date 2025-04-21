use bevy::prelude::*;

use crate::{
    components::RunwayComponent,
    resources::{Frame, PositionTransform, TransformationResource},
};

/// System to spawn the runway entity during startup if configured.
pub fn spawn_runway_sprite(
    mut commands: Commands,
    query: Query<(Entity, &RunwayComponent), Without<Sprite>>,
    transform_res: Res<TransformationResource>, // For coordinate conversion
) {
    for (entity, runway) in query.iter() {
        info!("Spawning runway entity...");

        // Convert NED position to screen coordinates
        let ned_pos = runway.position;
        let screen_pos = match transform_res.transform_to_screen_coords(&ned_pos, Frame::NED) {
            Ok(pos) => pos,
            Err(e) => {
                error!("Failed to transform runway NED position to screen: {}", e);
                // Default to origin if transform fails
                Vec3::ZERO
            }
        };

        // Convert NED heading (clockwise from North) to Bevy rotation (around Z, counter-clockwise)
        // NED North (0 deg) -> Bevy +Y axis
        // NED East (90 deg) -> Bevy +X axis
        // Rotation needs to align runway's local X (along centerline) with the heading direction in Bevy's space.
        // Bevy rotation is counter-clockwise. NED heading is clockwise.
        // A 0 deg NED heading means alignment with Bevy's +Y.
        // A 90 deg NED heading means alignment with Bevy's +X.
        // Rotation angle in Bevy = PI/2 - NED heading (in radians)
        let bevy_rotation_rad = std::f32::consts::FRAC_PI_2 - runway.heading as f32;
        let rotation = Quat::from_rotation_z(bevy_rotation_rad);

        // Calculate scale based on width/length and resource scale factor
        // let scale_factor = transform_res.get_scale() as f32; // meters per pixel -> use inverse? Check resource def. Assuming pixels = meters / scale_factor
        let scale_factor = 1.0;
        let render_width = runway.width as f32 / scale_factor;
        let render_length = runway.length as f32 / scale_factor;

        commands.entity(entity).insert((
            Sprite {
                color: Color::linear_rgb(0.2, 0.2, 0.2), // Dark grey color
                // Anchor might matter depending on how position is defined (threshold vs center)
                // Center is generally safer unless you offset the transform.
                anchor: bevy::sprite::Anchor::Center,
                custom_size: Some(Vec2::new(render_length, render_width)), // Map length/width
                ..default()
            },
            Transform {
                translation: screen_pos, // Use calculated screen position
                rotation,                // Use calculated rotation
                scale: Vec3::ONE,        // Default scale
            },
            // It's good practice to add these visibility/transform components
            // if inserting Transform after the initial spawn.
            GlobalTransform::default(),
            Visibility::Visible, // Ensure it's visible
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

        info!(
            "Runway spawned at {:?} (screen) with heading {:.1} deg",
            screen_pos,
            runway.heading.to_degrees()
        );
    }
}
