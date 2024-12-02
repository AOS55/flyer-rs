use crate::components::terrain::*;
use crate::components::CameraComponent;
use crate::ecs::{Result, System, World};
use crate::utils::constants::CHUNK_SIZE;
use glam::UVec2;
use std::collections::HashSet;

pub struct TerrainManagerSystem {
    loaded_chunks: HashSet<UVec2>,
}

impl TerrainManagerSystem {
    pub fn new() -> Self {
        Self {
            loaded_chunks: HashSet::new(),
        }
    }

    fn get_visible_chunks(world: &World) -> HashSet<UVec2> {
        let mut visible = HashSet::new();

        if let Ok(camera) = world.get_resource::<CameraComponent>() {
            // Calculate view distance based on viewport and zoom
            let viewport_extent = camera.viewport * 0.5 / camera.zoom;
            let view_distance = viewport_extent.length() / CHUNK_SIZE as f32;

            let chunk_pos = camera.position / CHUNK_SIZE as f32;
            let chunks_to_load = view_distance.ceil() as i32;

            // Add chunks in view distance
            for x in -chunks_to_load..=chunks_to_load {
                for y in -chunks_to_load..=chunks_to_load {
                    visible.insert(UVec2::new(
                        (chunk_pos.x + x as f32) as u32,
                        (chunk_pos.y + y as f32) as u32,
                    ));
                }
            }
        }

        visible
    }
}

impl System for TerrainManagerSystem {
    fn name(&self) -> &str {
        "Terrain Manager System"
    }

    fn run(&mut self, world: &mut World) -> Result<()> {
        let camera = world.get_resource::<CameraComponent>()?;
        let visible_chunks = TerrainComponent::get_visible_chunks(camera);

        for (_, terrain) in world.query_mut::<TerrainComponent>() {
            // Clean up chunks that are no longer visible
            let chunks_to_remove: Vec<UVec2> = terrain
                .chunks
                .keys()
                .filter(|pos| !visible_chunks.contains(pos))
                .copied()
                .collect();

            for pos in chunks_to_remove {
                terrain.chunks.remove(&pos);
            }

            // Update active chunks list
            terrain.active_chunks = visible_chunks.iter().copied().collect();
        }

        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["Camera System"]
    }
}

mod tests {
    use super::*;
    use crate::components::CameraComponent;
    use crate::ecs::EntityId;
    use glam::Vec2;

    fn setup_test_world() -> (World, EntityId) {
        let mut world = World::new();

        // Add camera
        let camera = CameraComponent {
            position: Vec2::ZERO,
            viewport: Vec2::new(800.0, 600.0),
            zoom: 1.0,
            ..Default::default()
        };
        world.add_resource(camera);

        // Add terrain component
        let terrain_entity = world.spawn();
        let terrain = TerrainComponent::new(
            UVec2::new(1000, 1000), // world size
            32,                     // chunk size
            12345,                  // seed
            1.0,                    // scale
        );
        world.add_component(terrain_entity, terrain).unwrap();

        (world, terrain_entity)
    }

    #[test]
    fn test_visible_chunks_calculation() {
        let (mut world, _) = setup_test_world();
        let system = TerrainManagerSystem::new();

        let visible_chunks = TerrainManagerSystem::get_visible_chunks(&world);
        assert!(!visible_chunks.is_empty());

        // Test chunk at camera position is visible
        let camera = world.get_resource::<CameraComponent>().unwrap();
        let center_chunk = UVec2::new(
            (camera.position.x / 32.0) as u32,
            (camera.position.y / 32.0) as u32,
        );
        assert!(visible_chunks.contains(&center_chunk));
    }

    #[test]
    fn test_chunk_cleanup() {
        let (mut world, terrain_entity) = setup_test_world();
        let mut system = TerrainManagerSystem::new();

        // Add some chunks
        let far_chunk = UVec2::new(100, 100);
        if let Ok(terrain) = world.get_component_mut::<TerrainComponent>(terrain_entity) {
            terrain.chunks.insert(far_chunk, TerrainChunk::new(32));
        }

        // Move camera far away
        if let Ok(mut camera) = world.get_resource_mut::<CameraComponent>() {
            camera.position = Vec2::new(0.0, 0.0);
        }

        // Run system
        system.run(&mut world).unwrap();

        // Check if far chunk was removed
        let terrain = world
            .get_component::<TerrainComponent>(terrain_entity)
            .unwrap();
        assert!(!terrain.chunks.contains_key(&far_chunk));
    }
}
