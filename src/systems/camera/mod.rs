mod controller;

pub use controller::CameraSystem;

use crate::components::{CameraComponent, SpatialComponent};
use crate::ecs::system::System;
use crate::resources::config::CameraConfig;

#[derive(Debug)]
pub struct CameraQuery {
    camera: CameraComponent,
    spatial: SpatialComponent,
}

#[derive(Debug, Clone, Copy)]
pub enum CameraMode {
    Free,
    Follow,
    FixedTarget,
    Orbit,
}
