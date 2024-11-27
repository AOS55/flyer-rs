pub mod asset;
pub mod core;
pub mod systems;

use std::path::PathBuf;
use std::sync::Arc;

use crate::utils::errors::SimError;
use crate::vehicles::Vehicle;

use self::asset::AssetManager;
use self::core::{SimulationState, WorldSettings};
use self::systems::System;

/// Main simulation world containing all simulation state and systems
pub struct SimWorld {
    state: SimulationState,
    settings: Arc<WorldSettings>,
    asset_manager: AssetManager,
    systems: Vec<Box<dyn System>>,
}

impl SimWorld {
    pub fn new(settings: WorldSettings) -> Result<Self, SimError> {
        let state = SimulationState::new(&settings)?;
        let asset_manager = AssetManager::new(
            &settings.asset_config.assets_path,
            &settings.asset_config.terrain_data_path,
        )?;

        Ok(Self {
            state,
            settings: Arc::new(settings),
            asset_manager,
            systems: Vec::new(),
        })
    }

    pub fn step(&mut self, dt: f64) -> Result<(), SimError> {
        // Update all registered systems
        for system in &mut self.systems {
            system.update(&mut self.state, dt)?;
        }

        // Update core simulation state
        self.state.step(dt)
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn add_vehicle(&mut self, vehicle: Box<dyn Vehicle>) {
        self.state.add_vehicle(vehicle);
    }

    pub fn settings(&self) -> Arc<WorldSettings> {
        self.settings.clone()
    }

    pub fn get_asset_manager(&mut self) -> &mut AssetManager {
        &mut self.asset_manager
    }

    pub fn set_assets_dir(&mut self, path: PathBuf) -> Result<(), SimError> {
        self.asset_manager.set_assets_path(path)
    }

    pub fn reset(&mut self) -> Result<(), SimError> {
        self.state = SimulationState::new(&self.settings)?;
        for system in &mut self.systems {
            system.reset();
        }
        Ok(())
    }
}
