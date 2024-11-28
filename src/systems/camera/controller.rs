use nalgebra::{Unit, UnitQuaternion, Vector3};
use std::f64::consts::PI;

use crate::components::{CameraComponent, SpatialComponent};
use crate::ecs::{
    component::ComponentManager,
    entity::EntityId,
    system::{System, SystemContext},
};
use crate::resources::config::CameraConfig;
use crate::utils::errors::SimError;

use super::CameraMode;

pub struct CameraSystem {
    mode: CameraMode,
    smooth_factor: f64,
    height_offset: f64,
    orbit_radius: f64,
    orbit_angle: f64,
}

impl CameraSystem {
    pub fn new(config: &CameraConfig) -> Self {
        Self {
            mode: CameraMode::Free,
            smooth_factor: config.smooth_factor,
            height_offset: config.height_offset,
            orbit_radius: config.orbit_radius,
            orbit_angle: 0.0,
        }
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
    }

    fn update_free_camera(
        &self,
        camera_entity: EntityId,
        components: &mut ComponentManager,
    ) -> Result<(), SimError> {
        if let (Some(camera), Some(spatial)) = (
            components.get_mut_component::<CameraComponent>(camera_entity),
            components.get_mut_component::<SpatialComponent>(camera_entity),
        ) {
            camera.update_view_matrix(&spatial.position, &spatial.attitude);
        }
        Ok(())
    }

    fn update_follow_camera(
        &self,
        camera_entity: EntityId,
        target_entity: EntityId,
        components: &mut ComponentManager,
        dt: f64,
    ) -> Result<(), SimError> {
        if let (Some(camera), Some(camera_spatial), Some(target_spatial)) = (
            components.get_mut_component::<CameraComponent>(camera_entity),
            components.get_mut_component::<SpatialComponent>(camera_entity),
            components.get_component::<SpatialComponent>(target_entity),
        ) {
            let target_pos = target_spatial.position;
            let target_att = target_spatial.attitude;

            let desired_pos =
                target_pos + (target_att * Vector3::new(0.0, 0.0, self.height_offset));
            let current_pos = &mut camera_spatial.position;

            *current_pos += (desired_pos - *current_pos) * self.smooth_factor * dt;
            camera.update_view_matrix(current_pos, &target_att);
        }
        Ok(())
    }

    fn update_fixed_target_camera(
        &self,
        camera_entity: EntityId,
        target_entity: EntityId,
        components: &mut ComponentManager,
    ) -> Result<(), SimError> {
        if let (Some(camera), Some(camera_spatial), Some(target_spatial)) = (
            components.get_mut_component::<CameraComponent>(camera_entity),
            components.get_mut_component::<SpatialComponent>(camera_entity),
            components.get_component::<SpatialComponent>(target_entity),
        ) {
            let look_dir = (target_spatial.position - camera_spatial.position).normalize();
            let up = Vector3::new(0.0, 0.0, 1.0);
            let camera_att = UnitQuaternion::face_towards(&look_dir, &up);

            camera.update_view_matrix(&camera_spatial.position, &camera_att);
        }
        Ok(())
    }

    fn update_orbit_camera(
        &mut self,
        camera_entity: EntityId,
        target_entity: EntityId,
        components: &mut ComponentManager,
        dt: f64,
    ) -> Result<(), SimError> {
        if let (Some(camera), Some(camera_spatial), Some(target_spatial)) = (
            components.get_mut_component::<CameraComponent>(camera_entity),
            components.get_mut_component::<SpatialComponent>(camera_entity),
            components.get_component::<SpatialComponent>(target_entity),
        ) {
            self.orbit_angle += dt * 0.5;
            if self.orbit_angle > 2.0 * PI {
                self.orbit_angle -= 2.0 * PI;
            }

            let orbit_pos = Vector3::new(
                self.orbit_radius * self.orbit_angle.cos(),
                self.orbit_radius * self.orbit_angle.sin(),
                self.height_offset,
            );

            camera_spatial.position = target_spatial.position + orbit_pos;
            let look_dir = (target_spatial.position - camera_spatial.position).normalize();
            let up = Vector3::new(0.0, 0.0, 1.0);
            let camera_att = UnitQuaternion::face_towards(&look_dir, &up);

            camera.update_view_matrix(&camera_spatial.position, &camera_att);
        }
        Ok(())
    }
}

impl System for CameraSystem {
    fn name(&self) -> &str {
        "Camera System"
    }

    fn update(&mut self, ctx: &mut SystemContext) -> Result<(), SimError> {
        for (camera_entity, camera) in ctx.components.query::<CameraComponent>() {
            match (self.mode, camera.target) {
                (CameraMode::Free, _) => {
                    self.update_free_camera(camera_entity, &mut ctx.components)?;
                }
                (CameraMode::Follow, Some(target)) => {
                    self.update_follow_camera(camera_entity, target, &mut ctx.components, ctx.dt)?;
                }
                (CameraMode::FixedTarget, Some(target)) => {
                    self.update_fixed_target_camera(camera_entity, target, &mut ctx.components)?;
                }
                (CameraMode::Orbit, Some(target)) => {
                    self.update_orbit_camera(camera_entity, target, &mut ctx.components, ctx.dt)?;
                }
                (_, None) => continue,
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.orbit_angle = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::ComponentManager;
    use crate::resources::config::CameraConfig;

    #[test]
    fn test_camera_system_creation() {
        let config = CameraConfig {
            smooth_factor: 0.1,
            height_offset: 10.0,
            orbit_radius: 20.0,
        };
        let system = CameraSystem::new(&config);
        assert_eq!(system.smooth_factor, 0.1);
        assert_eq!(system.height_offset, 10.0);
        assert_eq!(system.orbit_radius, 20.0);
    }

    #[test]
    fn test_camera_mode_switching() {
        let config = CameraConfig::default();
        let mut system = CameraSystem::new(&config);

        system.set_mode(CameraMode::Follow);
        match system.mode {
            CameraMode::Follow => (),
            _ => panic!("Camera mode not set correctly"),
        }
    }
}
