use super::System;
use crate::environment::terrain::{TerrainData, TerrainGenerator};
use crate::utils::errors::SimError;
use crate::world::asset::{AssetManager, LoadableAsset};
use crate::world::core::{Component, ComponentType, SimulationState};
use std::sync::Arc;

pub struct TerrainSystem {
    generator: TerrainGenerator,
    current_terrain: Option<Arc<TerrainData>>,
    needs_update: bool,
}

impl TerrainSystem {
    pub fn new(generator: TerrainGenerator) -> Self {
        Self {
            generator,
            current_terrain: None,
            needs_update: true,
        }
    }

    fn load_or_generate_terrain(
        &mut self,
        asset_manager: &mut AssetManager,
    ) -> Result<Arc<TerrainData>, SimError> {
        let terrain_name = self.generator.get_name();

        // Try to load from asset manager first
        if let Some(terrain) = asset_manager.get_terrain(&terrain_name)? {
            Ok(terrain)
        } else {
            // Generate new terrain if not found
            let terrain_data = self.generator.generate()?;
            let terrain = Arc::new(terrain_data);
            asset_manager.cache_terrain(&terrain_name, terrain.clone())?;
            Ok(terrain)
        }
    }

    pub fn mark_for_update(&mut self) {
        self.needs_update = true;
    }
}

impl System for TerrainSystem {
    fn update(&mut self, state: &mut SimulationState, _dt: f64) -> Result<(), SimError> {
        if self.needs_update {
            if let Some(asset_manager) = state.get_asset_manager() {
                let terrain = self.load_or_generate_terrain(asset_manager)?;

                // Update terrain component in state
                if let Some(terrain_component) =
                    state.get_component_mut(state.terrain_entity(), ComponentType::TERRAIN)
                {
                    if let Some(terrain_comp) = terrain_component
                        .as_any_mut()
                        .downcast_mut::<TerrainComponent>()
                    {
                        terrain_comp.update_terrain(terrain.clone());
                    }
                }

                self.current_terrain = Some(terrain);
                self.needs_update = false;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.current_terrain = None;
        self.needs_update = true;
    }
}

pub struct TerrainComponent {
    terrain: Option<Arc<TerrainData>>,
}

impl TerrainComponent {
    pub fn new() -> Self {
        Self { terrain: None }
    }

    pub fn update_terrain(&mut self, terrain: Arc<TerrainData>) {
        self.terrain = Some(terrain);
    }

    pub fn get_terrain(&self) -> Option<&Arc<TerrainData>> {
        self.terrain.as_ref()
    }
}

impl Component for TerrainComponent {
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
