use super::System;
use crate::utils::errors::SimError;
use crate::world::core::{Component, ComponentType, SimulationState};
use nalgebra::Vector3;

pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub fov: f32,
    follow_entity: Option<uuid::Uuid>,
}

impl Component for Camera {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn update(&mut self, _dt: f64) -> Result<(), SimError> {
        Ok(())
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            fov: 60.0,
            follow_entity: None,
        }
    }
}

pub struct CameraSystem {
    smooth_factor: f64,
    height_offset: f64,
}

impl CameraSystem {
    pub fn new(smooth_factor: f64, height_offset: f64) -> Self {
        Self {
            smooth_factor,
            height_offset,
        }
    }

    pub fn move_to(&mut self, camera: &mut Camera, position: Vector3<f64>) {
        let target_x = position.x;
        let target_y = position.y;
        let target_z = position.z + self.height_offset;

        camera.x += (target_x - camera.x) * self.smooth_factor;
        camera.y += (target_y - camera.y) * self.smooth_factor;
        camera.z += (target_z - camera.z) * self.smooth_factor;
    }

    pub fn set_follow_target(&mut self, camera: &mut Camera, entity_id: uuid::Uuid) {
        camera.follow_entity = Some(entity_id);
    }

    pub fn clear_follow_target(&mut self, camera: &mut Camera) {
        camera.follow_entity = None;
    }
}

impl System for CameraSystem {
    fn update(&mut self, state: &mut SimulationState, dt: f64) -> Result<(), SimError> {
        if let Some(camera) = state.get_component_mut(state.camera_entity(), ComponentType::CAMERA)
        {
            let camera = camera.as_any_mut().downcast_mut::<Camera>().unwrap();

            if let Some(entity_id) = camera.follow_entity {
                if let Some(component) =
                    state.get_component(entity_id.into(), ComponentType::VEHICLE)
                {
                    if let Some(position) = component.as_any().downcast_ref::<Vector3<f64>>() {
                        self.move_to(camera, *position);
                    }
                }
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        // Reset camera system state if needed
    }
}
