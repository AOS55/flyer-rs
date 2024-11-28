use crate::ecs::component::Component;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraComponent {
    pub position: Vector3<f64>,
    pub target: Option<Vector3<f64>>,
    pub up: Vector3<f64>,
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
    pub follow_mode: CameraFollowMode,
    pub smoothing_factor: f32,
    pub height_offset: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CameraFollowMode {
    Free,
    Follow {
        target_entity: u64,
        offset: Vector3<f64>,
    },
    LookAt {
        target_entity: u64,
        distance: f64,
    },
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 100.0),
            target: None,
            up: Vector3::z(),
            fov: 60.0,
            znear: 1.0,
            zfar: 1000.0,
            follow_mode: CameraFollowMode::Free,
            smoothing_factor: 0.1,
            height_offset: 20.0,
        }
    }
}

impl Component for CameraComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl CameraComponent {
    pub fn new(position: Vector3<f64>, fov: f32) -> Self {
        Self {
            position,
            fov,
            ..Default::default()
        }
    }

    pub fn with_follow_target(mut self, target_entity: u64, offset: Vector3<f64>) -> Self {
        self.follow_mode = CameraFollowMode::Follow {
            target_entity,
            offset,
        };
        self
    }

    pub fn with_look_at(mut self, target_entity: u64, distance: f64) -> Self {
        self.follow_mode = CameraFollowMode::LookAt {
            target_entity,
            distance,
        };
        self
    }

    pub fn set_free_mode(&mut self) {
        self.follow_mode = CameraFollowMode::Free;
        self.target = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = CameraComponent::default();
        assert_eq!(camera.position.z, 100.0);
        assert_eq!(camera.fov, 60.0);
    }

    #[test]
    fn test_camera_follow_modes() {
        let position = Vector3::new(0.0, 0.0, 10.0);
        let offset = Vector3::new(0.0, -10.0, 5.0);
        let camera = CameraComponent::new(position, 75.0).with_follow_target(1, offset);

        match camera.follow_mode {
            CameraFollowMode::Follow {
                target_entity,
                offset: follow_offset,
            } => {
                assert_eq!(target_entity, 1);
                assert_eq!(follow_offset, offset);
            }
            _ => panic!("Unexpected camera mode"),
        }
    }
}
