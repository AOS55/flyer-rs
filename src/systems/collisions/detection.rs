use bevy::prelude::*;
use nalgebra::Vector3;

use crate::{
    components::{
        terrain::TerrainChunkComponent, CollisionComponent, CollisionEvent, DubinsAircraftState,
        SpatialComponent,
    },
    resources::{AgentState, TerrainState},
    systems::{get_terrain_at_position, TerrainInfo},
};

pub fn collision_detection_system(
    mut dubins_collsion_query: Query<(Entity, &DubinsAircraftState, &mut CollisionComponent)>,
    mut spatial_collision_query: Query<(Entity, &SpatialComponent, &mut CollisionComponent)>,
    chunks: &Query<(&TerrainChunkComponent, &Transform)>,
    terrain_state: Res<TerrainState>,
    agent_state: Res<AgentState>, // used to access clock
    mut collision_events: EventWriter<CollisionEvent>,
) {
    // For Dubins aircraft
    for (entity, state, mut collision) in dubins_collsion_query.iter_mut() {
        if let Some(collision_info) = check_collision(
            entity,
            Vector3::new(
                state.spatial.position.x,
                state.spatial.position.y,
                state.spatial.position.z,
            ),
            &collision,
            &chunks,
            &terrain_state,
        ) {
            collision.register_collision(agent_state.sim_time);
            collision_events.send(collision_info);
        }
    }

    // For Full aircraft
    for (entity, spatial, mut collision) in spatial_collision_query.iter_mut() {
        if let Some(collision_info) = check_collision(
            entity,
            spatial.position,
            &collision,
            &chunks,
            &terrain_state,
        ) {
            collision.register_collision(agent_state.sim_time);
            collision_events.send(collision_info);
        }
    }
}

fn check_collision(
    entity: Entity,
    position: Vector3<f64>,
    collision: &CollisionComponent,
    chunks: &Query<(&TerrainChunkComponent, &Transform)>,
    terrain_state: &TerrainState,
) -> Option<CollisionEvent> {
    let aircraft_pos = position;
    let aircraft_height = -aircraft_pos.z - collision.height_offset;

    // Check center point
    let center_pos = Vec2::new(aircraft_pos.x as f32, aircraft_pos.y as f32);
    let center_info = get_terrain_at_position(center_pos, chunks, terrain_state)?;

    // Early exit if well above terrain
    if aircraft_height > center_info.height as f64 + collision.radius * 2.0 {
        return None;
    }

    let check_points = generate_check_points(position, collision.radius);
    let mut max_penetration = 0.0f64;
    let mut collision_point = position;
    let mut surface_normal = Vector3::new(0.0, 0.0, 1.0);

    for point in check_points {
        let point_pos = Vec2::new(point.x as f32, point.y as f32);
        if let Some(terrain_info) = get_terrain_at_position(point_pos, chunks, terrain_state) {
            let point_height = -point.z - collision.height_offset;
            let penetration = terrain_info.height as f64 - point_height;

            if penetration > 0.0 {
                // Update collision information if this is the deepest penetration
                if penetration > max_penetration {
                    max_penetration = penetration;
                    collision_point = point;

                    // Calculate approximate surface normal using neighboring points
                    surface_normal =
                        calculate_surface_normal(point_pos, &terrain_info, terrain_state, chunks);
                }
            }
        }
    }

    if max_penetration > 0.0 {
        Some(CollisionEvent {
            entity,
            impact_point: collision_point,
            normal: surface_normal,
            penetration_depth: max_penetration,
        })
    } else {
        None
    }
}

fn generate_check_points(center: Vector3<f64>, radius: f64) -> Vec<Vector3<f64>> {
    let mut points = vec![center]; // Center point

    // Add points in a circular pattern
    let num_radial_points = 8;
    for i in 0..num_radial_points {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (num_radial_points as f64);
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        points.push(Vector3::new(x, y, center.z));
    }

    points
}

fn calculate_surface_normal(
    pos: Vec2,
    terrain_info: &TerrainInfo,
    terrain_state: &TerrainState,
    chunks: &Query<(&TerrainChunkComponent, &Transform)>,
) -> Vector3<f64> {
    let dx = terrain_state.tile_size as f64;
    let dy = terrain_state.tile_size as f64;

    // Sample heights at neighboring points
    let right_pos = Vec2::new(pos.x + dx as f32, pos.y);
    let forward_pos = Vec2::new(pos.x, pos.y + dy as f32);

    let center_height = terrain_info.height as f64;
    let right_height = get_terrain_at_position(right_pos, chunks, terrain_state)
        .map(|info| info.height as f64)
        .unwrap_or(center_height);
    let forward_height = get_terrain_at_position(forward_pos, chunks, terrain_state)
        .map(|info| info.height as f64)
        .unwrap_or(center_height);

    // Calculate normal using cross product of terrain vectors
    let v1 = Vector3::new(dx, 0.0, right_height - center_height);
    let v2 = Vector3::new(0.0, dy, forward_height - center_height);
    let normal = v1.cross(&v2).normalize();

    normal
}
