use crate::components::CameraComponent;
use crate::ecs::{Result, System, World};

pub struct CameraControllerSystem {
    move_speed: f32,
    zoom_speed: f32,
    min_zoom: f32,
    max_zoom: f32,
}

impl CameraControllerSystem {
    pub fn new() -> Self {
        Self {
            move_speed: 500.0,
            zoom_speed: 0.1,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }

    pub fn get_viewport_bounds(camera: &CameraComponent) -> [f32; 4] {
        let half_size = camera.viewport * 0.5 / camera.zoom;
        [
            camera.position.x - half_size.x, // left
            camera.position.y - half_size.y, // top
            camera.position.x + half_size.x, // right
            camera.position.y + half_size.y, // bottom
        ]
    }

    fn update_position(&self, camera: &mut CameraComponent, _dt: f32) {
        if let Some(target) = camera.target {
            let diff = target - camera.position;
            let movement = diff * camera.interpolation_factor;
            camera.position += movement;
        }

        if let Some((min, max)) = camera.bounds {
            camera.position.x = camera.position.x.clamp(min.x, max.x);
            camera.position.y = camera.position.y.clamp(min.y, max.y);
        }
    }

    fn update_zoom(&self, camera: &mut CameraComponent, zoom_delta: f32) {
        let new_zoom =
            (camera.zoom + zoom_delta * self.zoom_speed).clamp(self.min_zoom, self.max_zoom);
        camera.zoom = new_zoom;
    }
}

impl System for CameraControllerSystem {
    fn name(&self) -> &str {
        "Camera Controller System"
    }

    fn run(&mut self, world: &mut World) -> Result<()> {
        let dt = world.get_resource::<f32>()?.clone();

        for (_, camera) in world.query_mut::<CameraComponent>() {
            self.update_position(camera, dt);
        }

        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{EntityId, World};
    use glam::Vec2;

    // Helper function to setup a basic world with camera
    fn setup_world() -> (World, EntityId) {
        let mut world = World::new();
        let camera_entity = world.spawn();
        let camera = CameraComponent::default();
        world.add_component(camera_entity, camera).unwrap();
        world.add_resource(1.0f32); // Add dt resource
        (world, camera_entity)
    }

    #[test]
    fn test_camera_component_default() {
        let camera = CameraComponent::default();
        assert_eq!(camera.position, Vec2::ZERO);
        assert_eq!(camera.viewport, Vec2::new(1920.0, 1080.0));
        assert_eq!(camera.zoom, 1.0);
        assert_eq!(camera.interpolation_factor, 0.1);
        assert!(camera.bounds.is_none());
        assert!(camera.target.is_none());
    }

    #[test]
    fn test_camera_follow_target() {
        let (mut world, camera_entity) = setup_world();
        let target_pos = Vec2::new(100.0, 100.0);
        let initial_pos = Vec2::ZERO;

        // Set target
        {
            let camera = world
                .get_component_mut::<CameraComponent>(camera_entity)
                .unwrap();
            camera.target = Some(target_pos);
            camera.position = initial_pos;
            camera.interpolation_factor = 0.5;
        }

        // Run system
        let mut system = CameraControllerSystem::new();
        system.run(&mut world).unwrap();

        // Check position update
        let camera = world
            .get_component::<CameraComponent>(camera_entity)
            .unwrap();
        assert_ne!(camera.position, initial_pos); // Verify movement occurred

        // Verify movement was in the right direction
        let to_target = target_pos - initial_pos;
        let movement = camera.position - initial_pos;
        assert!(to_target.dot(movement) > 0.0); // Verify we moved toward the target
    }

    #[test]
    fn test_camera_bounds() {
        let (mut world, camera_entity) = setup_world();
        let min_bounds = Vec2::new(-100.0, -100.0);
        let max_bounds = Vec2::new(100.0, 100.0);

        // Set bounds and position beyond bounds
        {
            let camera = world
                .get_component_mut::<CameraComponent>(camera_entity)
                .unwrap();
            camera.bounds = Some((min_bounds, max_bounds));
            camera.position = Vec2::new(200.0, 200.0); // Outside bounds
        }

        // Run system
        let mut system = CameraControllerSystem::new();
        system.run(&mut world).unwrap();

        // Check position is clamped
        let camera = world
            .get_component::<CameraComponent>(camera_entity)
            .unwrap();
        assert_eq!(camera.position, Vec2::new(100.0, 100.0)); // Should be clamped to max bounds
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = CameraComponent::default();
        let system = CameraControllerSystem::new();

        // Test zoom in
        system.update_zoom(&mut camera, 1.0);
        assert!(camera.zoom > 1.0);
        assert!(camera.zoom <= system.max_zoom);

        // Test zoom out
        system.update_zoom(&mut camera, -2.0);
        assert!(camera.zoom >= system.min_zoom);
    }

    #[test]
    fn test_camera_system_dependencies() {
        let system = CameraControllerSystem::new();
        assert!(system.dependencies().is_empty());
    }
}
