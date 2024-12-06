use crate::components::FlightCamera;
use crate::components::Player;
use bevy::input::mouse::MouseWheel;
use bevy::math::Vec2;
use bevy::prelude::*;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d::default(), FlightCamera::default()));
}

pub fn camera_follow_system(
    mut camera_query: Query<(&FlightCamera, &mut Transform, &mut OrthographicProjection)>,
    time: Res<Time>,
) {
    for (camera, mut transform, mut projection) in camera_query.iter_mut() {
        // Update position if there's a target
        if let Some(target) = camera.target {
            let current_pos = transform.translation.truncate();
            let diff = target - current_pos;
            let movement = diff * camera.interpolation_factor * time.delta_secs();
            transform.translation += movement.extend(0.0);

            // Apply bounds if they exist
            if let Some((min, max)) = camera.bounds {
                transform.translation.x = transform.translation.x.clamp(min.x, max.x);
                transform.translation.y = transform.translation.y.clamp(min.y, max.y);
            }
        }
    }
}

pub fn camera_zoom_system(
    mut camera_query: Query<(&FlightCamera, &mut OrthographicProjection)>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    let scroll = scroll_evr.read().map(|ev| ev.y).sum::<f32>();
    if scroll == 0.0 {
        return;
    }

    for (camera, mut projection) in camera_query.iter_mut() {
        let zoom_delta = -scroll * camera.zoom_speed;
        projection.scale = (projection.scale + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
    }
}

pub fn update_camera_target(
    mut camera_query: Query<&mut FlightCamera>,
    target_query: Query<&Transform, (With<Player>, Without<FlightCamera>)>,
) {
    if let Ok(mut camera) = camera_query.get_single_mut() {
        if let Ok(target_transform) = target_query.get_single() {
            camera.target = Some(target_transform.translation.truncate());
        }
    }
}
