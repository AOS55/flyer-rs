use bevy::prelude::*;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use crate::components::AircraftGeometry;

#[derive(Event)]
pub struct CollisionEvent {
    /// Entity that collided
    pub entity: Entity,
    /// Point of impact in space
    pub impact_point: Vector3<f64>,
    /// Surface normal at impact point
    pub normal: Vector3<f64>,
    /// Penetration depth
    pub penetration_depth: f64,
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CollisionComponent {
    /// Vertical offset from center for collision checks
    pub height_offset: f64,
    /// Radius for horizontal collision checks
    pub radius: f64,
    /// Whether entity has collided this frame
    #[serde(skip)]
    pub has_collided: bool,
    /// Collision metrics for the current episode
    #[serde(skip)]
    pub collision_count: u32,
    /// Time of last collision
    #[serde(skip)]
    pub last_collision_time: f64,
}

impl Default for CollisionComponent {
    fn default() -> Self {
        Self {
            height_offset: 2.0,
            radius: 20.0,
            has_collided: false,
            collision_count: 0,
            last_collision_time: 0.0,
        }
    }
}

impl CollisionComponent {
    pub fn new(height_offset: f64, radius: f64) -> Self {
        Self {
            height_offset,
            radius,
            has_collided: false,
            collision_count: 0,
            last_collision_time: 0.0,
        }
    }

    pub fn from_geometry(geometry: &AircraftGeometry) -> Self {
        Self {
            height_offset: geometry.wing_span / 10.0,
            radius: geometry.wing_span / 2.0,
            has_collided: false,
            collision_count: 0,
            last_collision_time: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.has_collided = false;
        self.collision_count = 0;
        self.last_collision_time = 0.0;
    }

    pub fn register_collision(&mut self, time: f64) {
        self.has_collided = true;
        self.collision_count += 1;
        self.last_collision_time = time;
    }

    pub fn has_recent_collision(&self, current_time: f64, window: f64) -> bool {
        self.has_collided && (current_time - self.last_collision_time) < window
    }
}
