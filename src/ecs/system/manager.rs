use super::{ComponentAccess, System, SystemId};
use crate::ecs::error::{EcsError, Result};
use crate::ecs::World;
use std::collections::{HashMap, HashSet};

pub struct SystemDescriptor {
    system: Box<dyn System>,
    access: ComponentAccess,
    dependencies: Vec<SystemId>,
}

pub struct SystemManager {
    systems: HashMap<SystemId, SystemDescriptor>,
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

        let descriptor = SystemDescriptor {
            access: system.component_access(),
            dependencies: Vec::new(),
            system,
        };

        self.systems.insert(id, descriptor);
        id
    }

    pub fn add_dependency(&mut self, system_id: SystemId, depends_on: SystemId) -> Result<()> {
        if !self.systems.contains_key(&system_id) || !self.systems.contains_key(&depends_on) {
            return Err(EcsError::SystemError("Invalid system ID".to_string()));
        }

        if let Some(descriptor) = self.systems.get_mut(&system_id) {
            descriptor.dependencies.push(depends_on);
        }

        Ok(())
    }

    pub fn run_systems(&mut self, world: &mut World) -> Result<()> {
        // Create execution groups based on access patterns
        let execution_groups = self.create_execution_groups();

        // Run each group sequentially, systems within groups can potentially run in parallel
        for group in execution_groups {
            for system_id in group {
                if let Some(descriptor) = self.systems.get_mut(&system_id) {
                    descriptor.system.run(world).map_err(|e| {
                        EcsError::SystemError(format!(
                            "System '{}' failed: {}",
                            descriptor.system.name(),
                            e
                        ))
                    })?;
                }
            }
        }
        Ok(())
    }

    fn create_execution_groups(&self) -> Vec<Vec<SystemId>> {
        let mut groups = Vec::new();
        let mut remaining: HashSet<_> = self.systems.keys().copied().collect();

        while !remaining.is_empty() {
            let mut current_group = Vec::new();
            let mut next_remaining = remaining.clone();

            for &system_id in &remaining {
                if let Some(system) = self.systems.get(&system_id) {
                    // Check if this system can run with current group
                    if current_group.iter().all(|&other_id| {
                        self.systems
                            .get(&other_id)
                            .map_or(true, |other| !system.access.conflicts_with(&other.access))
                    }) {
                        current_group.push(system_id);
                        next_remaining.remove(&system_id);
                    }
                }
            }

            if !current_group.is_empty() {
                groups.push(current_group);
            }
            remaining = next_remaining;
        }

        groups
    }

    pub fn get_system(&self, id: SystemId) -> Option<&(dyn System + '_)> {
        self.systems
            .get(&id)
            .map(|descriptor| descriptor.system.as_ref())
    }

    pub fn get_system_mut(&mut self, id: SystemId) -> Option<&mut (dyn System + '_)> {
        self.systems
            .get_mut(&id)
            .map(|descriptor| descriptor.system.as_mut())
    }

    pub fn run_single_system(&mut self, id: SystemId, world: &mut World) -> Result<()> {
        if let Some(descriptor) = self.systems.get_mut(&id) {
            descriptor.system.run(world)
        } else {
            Ok(())
        }
    }
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}
