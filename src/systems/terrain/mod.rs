mod generator;

use crate::components::{SpatialComponent, TerrainComponent};
use crate::ecs::{System, World};
use crate::resources::config::TerrainConfig;
use crate::utils::errors::SimError;

pub use generator::TerrainGenerator;

pub struct TerrainSystem {
    generator: TerrainGenerator,
    update_required: bool,
}

impl TerrainSystem {
    pub fn new(config: TerrainConfig) -> Self {
        Self {
            generator: TerrainGenerator::new(config),
            update_required: true,
        }
    }

    pub fn mark_for_update(&mut self) {
        self.update_required = true;
    }
}

impl System for TerrainSystem {
    fn update(&mut self, world: &mut World, _dt: f64) -> Result<(), SimError> {
        if !self.update_required {
            return Ok(());
        }

        for (entity, (terrain, spatial)) in
            world.query_mut::<(&mut TerrainComponent, &mut SpatialComponent)>()
        {
            if let Some(new_data) = self.generator.generate_chunk(spatial.position)? {
                terrain.update_data(new_data);
            }

            let config = world.get_resource::<TerrainConfig>()?;
            if let Some(height) = terrain.get_height_at(spatial.position) {
                if spatial.position.z < height {
                    spatial.position.z = height;
                }
            }
        }

        self.update_required = false;
        Ok(())
    }

    fn reset(&mut self) {
        self.update_required = true;
    }
}
