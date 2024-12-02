use crate::components::{CameraComponent, RenderComponent};
use crate::ecs::{EcsError, Result, System, World};
use crate::resources::{AssetManager, RenderConfig, TimeManager};
use crate::systems::camera::CameraControllerSystem;
use glam::{Vec2, Vec4};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use tiny_skia::{BlendMode, Color, Pixmap, PixmapPaint, Transform};

/// Represents a single render command in the render queue
#[derive(Debug, Clone)]
struct RenderCommand {
    texture_id: String,
    position: Vec2,
    scale: Vec2,
    rotation: f32,
    layer: i32,
    tint: Vec4,
    src_rect: Option<[f32; 4]>, // For sprite sheets: [x, y, w, h]
    flip: (bool, bool),         // (flip_x, flip_y)
}

impl Ord for RenderCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by layer first, then by Y position for same layer
        self.layer.cmp(&other.layer).then_with(|| {
            self.position
                .y
                .partial_cmp(&other.position.y)
                .unwrap_or(Ordering::Equal)
        })
    }
}

impl PartialOrd for RenderCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RenderCommand {
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer && self.position.y == other.position.y
    }
}

impl Eq for RenderCommand {}

pub struct RenderSystem {
    render_commands: BinaryHeap<RenderCommand>,
    render_buffer: Pixmap,
    temp_buffer: Pixmap, // For flip operations
    viewport_size: Vec2,
    clear_color: Color,
}

impl RenderSystem {
    pub fn new(config: &RenderConfig) -> Self {
        let render_buffer = Pixmap::new(config.screen_width, config.screen_height)
            .expect("Failed to create render buffer");
        let temp_buffer = Pixmap::new(config.screen_width, config.screen_height)
            .expect("Failed to create temp buffer");

        Self {
            render_commands: BinaryHeap::new(),
            render_buffer,
            temp_buffer,
            viewport_size: Vec2::new(config.screen_width as f32, config.screen_height as f32),
            clear_color: Color::from_rgba8(0, 0, 0, 255),
        }
    }

    fn is_visible(&self, position: Vec2, scale: Vec2, camera: &CameraComponent) -> bool {
        let bounds = CameraControllerSystem::get_viewport_bounds(camera);
        let half_size = scale * 0.5;
        let min = position - half_size;
        let max = position + half_size;

        min.x <= bounds[2] && max.x >= bounds[0] && min.y <= bounds[3] && max.y >= bounds[1]
    }

    fn world_to_screen(&self, pos: Vec2, camera: &CameraComponent) -> Vec2 {
        let camera_pos = camera.position;
        let zoom = camera.zoom;
        let centered = pos - camera_pos;
        let scaled = centered * zoom;
        let screen_center = self.viewport_size * 0.5;
        screen_center + scaled
    }

    fn gather_render_commands(&mut self, world: &World) -> Result<()> {
        self.render_commands.clear();

        // Get camera using query, still need to handle Option here
        let camera = world
            .query::<CameraComponent>()
            .next()
            .map(|(_, camera)| camera)
            .ok_or_else(|| EcsError::SystemError("No camera found".to_string()))?;

        // Gather all visible renderable entities
        for (_, render) in world.query::<RenderComponent>() {
            if !render.visible {
                continue;
            }

            if !self.is_visible(render.position, render.scale, camera) {
                continue;
            }

            self.render_commands.push(RenderCommand {
                texture_id: render.texture_id.clone(),
                position: self.world_to_screen(render.position, camera),
                scale: render.scale * camera.zoom,
                rotation: render.rotation,
                layer: render.layer,
                tint: render.tint,
                src_rect: render.src_rect,
                flip: (render.flip_x, render.flip_y),
            });
        }
        Ok(())
    }

    fn render_sprite(&mut self, texture: &Pixmap, cmd: &RenderCommand) {
        let mut paint = PixmapPaint::default();
        paint.blend_mode = BlendMode::SourceOver;

        // Create transform matrix
        let transform = Transform::from_row(
            1.0,
            0.0,
            cmd.position.x - (cmd.scale.x * 0.5),
            0.0,
            1.0,
            cmd.position.y - (cmd.scale.y * 0.5),
        )
        .pre_scale(
            cmd.scale.x / texture.width() as f32,
            cmd.scale.y / texture.height() as f32,
        )
        .pre_rotate(cmd.rotation);

        // Handle flipping using temp buffer if needed
        if cmd.flip.0 || cmd.flip.1 {
            self.temp_buffer.fill(Color::TRANSPARENT);
            let flip_transform = Transform::from_scale(
                if cmd.flip.0 { -1.0 } else { 1.0 },
                if cmd.flip.1 { -1.0 } else { 1.0 },
            );
            self.temp_buffer
                .draw_pixmap(0, 0, texture.as_ref(), &paint, flip_transform, None);
            self.render_buffer.draw_pixmap(
                0,
                0,
                self.temp_buffer.as_ref(),
                &paint,
                transform,
                None,
            );
        } else {
            self.render_buffer
                .draw_pixmap(0, 0, texture.as_ref(), &paint, transform, None);
        }
    }

    fn execute_render_commands(&mut self, world: &World) -> Result<()> {
        // Get AssetManager - already returns a Result
        let assets = world.get_resource::<AssetManager>()?;

        // Clear screen
        self.render_buffer.fill(self.clear_color);

        // Execute all render commands in order
        while let Some(cmd) = self.render_commands.pop() {
            if let Some(texture) = assets.get_texture(&cmd.texture_id) {
                self.render_sprite(&texture, &cmd);
            } else {
                // Print missing texture but continue rendering
                println!("Texture not found: {}", cmd.texture_id);
            }
        }
        Ok(())
    }

    pub fn get_render_buffer(&self) -> &Pixmap {
        &self.render_buffer
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.render_buffer = Pixmap::new(width, height).expect("Failed to resize render buffer");
        self.temp_buffer = Pixmap::new(width, height).expect("Failed to resize temp buffer");
        self.viewport_size = Vec2::new(width as f32, height as f32);
    }
}

impl System for RenderSystem {
    fn name(&self) -> &str {
        "Render System"
    }

    fn run(&mut self, world: &mut World) -> Result<()> {
        // Get TimeManager - already returns a Result
        // world.get_resource::<TimeManager>()?;

        // Gather and execute render commands
        self.gather_render_commands(world)?;
        self.execute_render_commands(world)?;
        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["Camera System"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::CameraComponent;
    use crate::ecs::World;
    use crate::resources::{AssetManager, RenderConfig, TimeManager};
    use glam::{Vec2, Vec4};
    use std::path::PathBuf;

    fn create_test_render_command(layer: i32) -> RenderCommand {
        RenderCommand {
            texture_id: "test_texture".to_string(),
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
            layer,
            tint: Vec4::ONE,
            src_rect: None,
            flip: (false, false),
        }
    }

    fn setup_test_world() -> (World, RenderSystem) {
        let mut world = World::new();

        // Add required resources
        let render_config = RenderConfig {
            screen_width: 800,
            screen_height: 600,
            vsync: true,
            fov: 60.0,
            draw_distance: 1000.0,
        };

        let asset_manager = AssetManager::new(PathBuf::from("test_assets"));
        let time_manager = TimeManager::new();

        world.add_resource(render_config.clone()).unwrap();
        world.add_resource(asset_manager).unwrap();
        world.add_resource(time_manager).unwrap();

        let render_system = RenderSystem::new(&render_config);

        // Add default camera
        let camera_entity = world.spawn();
        world
            .add_component(
                camera_entity,
                CameraComponent {
                    position: Vec2::ZERO,
                    zoom: 1.0,
                    ..Default::default()
                },
            )
            .unwrap();

        (world, render_system)
    }

    #[test]
    fn test_render_system_creation() {
        let config = RenderConfig::default();
        let system = RenderSystem::new(&config);

        assert_eq!(system.viewport_size.x, config.screen_width as f32);
        assert_eq!(system.viewport_size.y, config.screen_height as f32);
    }

    #[test]
    fn test_render_command_ordering() {
        let mut heap = std::collections::BinaryHeap::new();

        // Create commands with different layers
        let cmd1 = create_test_render_command(0);
        let cmd2 = create_test_render_command(1);
        let cmd3 = create_test_render_command(2);

        heap.push(cmd1.clone());
        heap.push(cmd2.clone());
        heap.push(cmd3.clone());

        // Commands should come out in reverse layer order (highest first)
        assert_eq!(heap.pop().unwrap().layer, 2);
        assert_eq!(heap.pop().unwrap().layer, 1);
        assert_eq!(heap.pop().unwrap().layer, 0);
    }

    #[test]
    fn test_render_command_same_layer_ordering() {
        let mut heap = std::collections::BinaryHeap::new();

        // Create commands with same layer but different Y positions
        let mut cmd1 = create_test_render_command(0);
        let mut cmd2 = create_test_render_command(0);

        cmd1.position.y = 0.0;
        cmd2.position.y = 10.0;

        heap.push(cmd1);
        heap.push(cmd2);

        // Higher Y position should come first
        assert_eq!(heap.pop().unwrap().position.y, 10.0);
        assert_eq!(heap.pop().unwrap().position.y, 0.0);
    }

    #[test]
    fn test_world_to_screen_conversion() {
        let (_, system) = setup_test_world();

        let camera = CameraComponent {
            position: Vec2::new(100.0, 100.0),
            zoom: 2.0,
            ..Default::default()
        };

        let world_pos = Vec2::new(150.0, 150.0);
        let screen_pos = system.world_to_screen(world_pos, &camera);

        let expected_pos = system.viewport_size * 0.5 + (world_pos - camera.position) * camera.zoom;
        assert!((screen_pos - expected_pos).length() < 0.001);
    }

    #[test]
    fn test_render_buffer_size() {
        let config = RenderConfig::default();
        let system = RenderSystem::new(&config);

        assert_eq!(system.render_buffer.width(), config.screen_width);
        assert_eq!(system.render_buffer.height(), config.screen_height);
    }

    #[test]
    fn test_visibility_check() {
        let (_, system) = setup_test_world();

        let camera = CameraComponent {
            position: Vec2::ZERO,
            zoom: 1.0,
            ..Default::default()
        };

        // Test object in view
        assert!(system.is_visible(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0), &camera));

        // Test object out of view
        assert!(!system.is_visible(Vec2::new(1000.0, 1000.0), Vec2::new(10.0, 10.0), &camera));
    }

    #[test]
    fn test_resize() {
        let (_, mut system) = setup_test_world();

        let new_width = 1024;
        let new_height = 768;

        system.resize(new_width, new_height);

        assert_eq!(system.viewport_size.x, new_width as f32);
        assert_eq!(system.viewport_size.y, new_height as f32);
        assert_eq!(system.render_buffer.width(), new_width);
        assert_eq!(system.render_buffer.height(), new_height);
        assert_eq!(system.temp_buffer.width(), new_width);
        assert_eq!(system.temp_buffer.height(), new_height);
    }

    #[test]
    fn test_gather_render_commands() {
        let (mut world, mut system) = setup_test_world();

        // Create test entities with render components
        let entity1 = world.spawn();
        let entity2 = world.spawn();

        world
            .add_component(
                entity1,
                RenderComponent {
                    texture_id: "texture1".to_string(),
                    position: Vec2::new(0.0, 0.0),
                    scale: Vec2::ONE,
                    rotation: 0.0,
                    layer: 0,
                    visible: true,
                    tint: Vec4::ONE,
                    src_rect: None,
                    flip_x: false,
                    flip_y: false,
                },
            )
            .unwrap();

        world
            .add_component(
                entity2,
                RenderComponent {
                    texture_id: "texture2".to_string(),
                    position: Vec2::new(10.0, 10.0),
                    layer: 1,
                    visible: true,
                    ..Default::default()
                },
            )
            .unwrap();

        let _ = system.gather_render_commands(&world);

        // Check if commands were gathered and ordered correctly
        let commands: Vec<_> = system.render_commands.into_sorted_vec();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].layer, 0);
        assert_eq!(commands[1].layer, 1);
    }

    #[test]
    fn test_invisible_entities_not_rendered() {
        let (mut world, mut system) = setup_test_world();

        let entity = world.spawn();
        world
            .add_component(
                entity,
                RenderComponent {
                    texture_id: "test_texture".to_string(),
                    visible: false,
                    ..Default::default()
                },
            )
            .unwrap();

        let _ = system.gather_render_commands(&world);
        assert!(
            system.render_commands.is_empty(),
            "Invisible entities should not generate render commands"
        );
    }

    #[test]
    fn test_camera_zoom_affects_visibility() {
        let (mut world, mut system) = setup_test_world();

        // Update camera with very small zoom
        let camera_entity = world.spawn();
        world
            .add_component(
                camera_entity,
                CameraComponent {
                    position: Vec2::ZERO,
                    zoom: 0.1, // Very zoomed out
                    ..Default::default()
                },
            )
            .unwrap();

        let entity = world.spawn();
        world
            .add_component(
                entity,
                RenderComponent {
                    position: Vec2::new(1000.0, 1000.0),
                    scale: Vec2::new(10.0, 10.0),
                    ..Default::default()
                },
            )
            .unwrap();

        let _ = system.gather_render_commands(&world);
        assert!(
            system.render_commands.is_empty(),
            "Entities should not be visible when zoomed out too far"
        );
    }

    #[test]
    fn test_handle_missing_camera() {
        let mut world = World::new();
        let config = RenderConfig::default();
        let mut system = RenderSystem::new(&config);

        // Add required resources but no camera
        world
            .add_resource(AssetManager::new(PathBuf::from("test_assets")))
            .unwrap();
        world.add_resource(TimeManager::new()).unwrap();

        // This should handle the missing camera gracefully
        let result = system.run(&mut world);
        assert!(
            result.is_err(),
            "System should error when no camera is present"
        );
    }

    #[test]
    fn test_layer_overflow() {
        let (mut world, mut system) = setup_test_world();

        // Add entities with extreme layer values
        let entity1 = world.spawn();
        let entity2 = world.spawn();

        world
            .add_component(
                entity1,
                RenderComponent {
                    layer: i32::MAX,
                    ..Default::default()
                },
            )
            .unwrap();

        world
            .add_component(
                entity2,
                RenderComponent {
                    layer: i32::MIN,
                    ..Default::default()
                },
            )
            .unwrap();

        let _ = system.gather_render_commands(&world);
        let commands: Vec<_> = system.render_commands.into_sorted_vec();

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].layer, i32::MIN);
        assert_eq!(commands[1].layer, i32::MAX);
    }

    #[test]
    fn test_multiple_entities_same_position() {
        let (mut world, mut system) = setup_test_world();

        // Create multiple entities at the same position but different layers
        let position = Vec2::new(100.0, 100.0);
        for i in 0..5 {
            let entity = world.spawn();
            world
                .add_component(
                    entity,
                    RenderComponent {
                        position,
                        layer: i,
                        ..Default::default()
                    },
                )
                .unwrap();
        }

        let _ = system.gather_render_commands(&world);
        let commands: Vec<_> = system.render_commands.into_sorted_vec();

        // Verify ordering is maintained even with same positions
        for i in 1..commands.len() {
            assert!(
                commands[i].layer > commands[i - 1].layer,
                "Render commands should maintain layer order with same positions"
            );
        }
    }

    #[test]
    fn test_extreme_scale_values() {
        let (mut world, mut system) = setup_test_world();

        let entity = world.spawn();
        world
            .add_component(
                entity,
                RenderComponent {
                    scale: Vec2::new(f32::MAX, f32::MAX),
                    ..Default::default()
                },
            )
            .unwrap();

        // Should handle extreme scales gracefully
        let _ = system.gather_render_commands(&world);
    }

    #[test]
    fn test_sprite_flipping() {
        let (mut world, mut system) = setup_test_world();

        let entity = world.spawn();
        world
            .add_component(
                entity,
                RenderComponent {
                    flip_x: true,
                    flip_y: true,
                    ..Default::default()
                },
            )
            .unwrap();

        let _ = system.gather_render_commands(&world);
        let cmd = system.render_commands.pop().unwrap();
        assert_eq!(
            cmd.flip,
            (true, true),
            "Flip flags should be properly transferred to render command"
        );
    }

    #[test]
    fn test_empty_world_handling() {
        let (world, mut system) = setup_test_world();

        let _ = system.gather_render_commands(&world);
        assert!(
            system.render_commands.is_empty(),
            "Empty world should result in no render commands"
        );
    }

    #[test]
    fn test_resource_cleanup() {
        let config = RenderConfig::default();
        let system = RenderSystem::new(&config);

        // Verify buffers are properly allocated
        assert!(!system.render_buffer.data().is_empty());
        assert!(!system.temp_buffer.data().is_empty());

        // System should clean up resources when dropped
        drop(system);
        // Note: We can't directly test if resources are freed, but we can ensure
        // the drop implementation completes without panicking
    }

    #[test]
    fn test_concurrent_buffer_access() {
        use std::sync::Arc;
        use std::thread;

        let (_, system) = setup_test_world();
        let system = Arc::new(std::sync::Mutex::new(system));

        // Simulate multiple threads trying to access the render buffer
        let mut handles = vec![];
        for _ in 0..4 {
            let system_clone = Arc::clone(&system);
            let handle = thread::spawn(move || {
                let system = system_clone.lock().unwrap();
                let buffer = system.get_render_buffer();
                assert!(!buffer.data().is_empty());
            });
            handles.push(handle);
        }

        // Ensure all threads complete without deadlock or panic
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
