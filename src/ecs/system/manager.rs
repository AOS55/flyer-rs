use super::{System, SystemId};
use crate::ecs::error::{EcsError, Result};
use crate::ecs::World;
use std::collections::HashMap;

pub struct SystemManager {
    systems: HashMap<SystemId, Box<dyn System>>,
    next_id: usize,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn insert_system(&mut self, system: Box<dyn System>) -> SystemId {
        let id = SystemId(self.next_id);
        self.next_id += 1;
        self.systems.insert(id, system);
        id
    }

    pub fn remove_system(&mut self, id: SystemId) -> Option<Box<dyn System>> {
        self.systems.remove(&id)
    }

    pub fn get_system(&self, id: SystemId) -> Option<&dyn System> {
        self.systems.get(&id).map(|s| s.as_ref())
    }

    pub fn get_system_mut(&mut self, id: SystemId) -> Option<&mut (dyn System + '_)> {
        if let Some(system) = self.systems.get_mut(&id) {
            Some(system.as_mut())
        } else {
            None
        }
    }

    pub fn run_system(&mut self, id: SystemId, world: &mut World) -> Result<()> {
        if let Some(system) = self.systems.get_mut(&id) {
            system.run(world).map_err(|e| {
                EcsError::SystemError(format!("System '{}' failed: {}", system.name(), e))
            })?;
        }
        Ok(())
    }

    pub fn run_systems(&mut self, world: &mut World) -> Result<()> {
        // Create a vector of system IDs to avoid borrowing issues
        let system_ids: Vec<SystemId> = self.systems.keys().copied().collect();

        // Run each system
        for id in system_ids {
            if let Some(system) = self.systems.get_mut(&id) {
                system.run(world).map_err(|e| {
                    EcsError::SystemError(format!("System '{}' failed: {}", system.name(), e))
                })?;
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SystemId, &dyn System)> + '_ {
        self.systems
            .iter()
            .map(|(id, system)| (*id, system.as_ref()))
    }
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}
