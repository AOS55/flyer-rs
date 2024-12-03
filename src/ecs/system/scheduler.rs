use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use super::{ComponentAccess, SystemId, SystemManager};
use crate::ecs::error::{EcsError, Result};
use crate::ecs::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    PreUpdate,
    Update,
    PostUpdate,
}

pub struct SystemScheduler {
    systems: HashMap<SystemId, Stage>,
    dependency_graph: Graph<SystemId, ()>,
    node_indices: HashMap<SystemId, NodeIndex>,
    access_patterns: HashMap<SystemId, ComponentAccess>,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            dependency_graph: Graph::new(),
            node_indices: HashMap::new(),
            access_patterns: HashMap::new(),
        }
    }

    pub fn add_system(
        &mut self,
        id: SystemId,
        stage: Stage,
        access: ComponentAccess,
        dependencies: Vec<SystemId>,
    ) {
        self.systems.insert(id, stage);
        self.access_patterns.insert(id, access);

        let node_idx = self.dependency_graph.add_node(id);
        self.node_indices.insert(id, node_idx);

        for dep_id in dependencies {
            if let Some(&dep_idx) = self.node_indices.get(&dep_id) {
                self.dependency_graph.add_edge(dep_idx, node_idx, ());
            }
        }
    }

    pub fn build_execution_order(&self) -> Vec<Vec<SystemId>> {
        let mut stages = HashMap::new();
        for (id, &stage) in &self.systems {
            stages.entry(stage).or_insert_with(HashSet::new).insert(*id);
        }

        let mut execution_order = Vec::new();
        let system_order = toposort(&self.dependency_graph, None).unwrap_or_default();

        for &stage in &[Stage::PreUpdate, Stage::Update, Stage::PostUpdate] {
            let mut stage_systems = Vec::new();
            let mut current_group = Vec::new();

            for node_idx in &system_order {
                let system_id = self.dependency_graph[*node_idx];
                if !stages.get(&stage).map_or(false, |s| s.contains(&system_id)) {
                    continue;
                }

                let current_access = &self.access_patterns[&system_id];

                // Check if system can run with current group
                if current_group.iter().any(|&other_id| {
                    current_access.conflicts_with(&self.access_patterns[&other_id])
                }) {
                    // If conflicts exist, start new group
                    if !current_group.is_empty() {
                        stage_systems.push(current_group);
                        current_group = Vec::new();
                    }
                }

                current_group.push(system_id);
            }

            if !current_group.is_empty() {
                stage_systems.push(current_group);
            }

            if !stage_systems.is_empty() {
                execution_order.extend(stage_systems);
            }
        }

        execution_order
    }

    pub fn remove_system(&mut self, id: SystemId) {
        self.systems.remove(&id);
        self.access_patterns.remove(&id);
        if let Some(idx) = self.node_indices.remove(&id) {
            self.dependency_graph.remove_node(idx);
        }
    }

    pub fn execute_systems(
        &self,
        world: World,
        manager: SystemManager, // Take ownership of both World and SystemManager
    ) -> Result<(World, SystemManager)> {
        // Return both modified structures
        let execution_order = self.build_execution_order();

        let mut current_world = world;
        let mut current_manager = manager;

        // Execute each group of systems sequentially
        for group in execution_order {
            // Wrap both world and manager in Arc<Mutex> for safe parallel access
            let world_mutex = Arc::new(Mutex::new(current_world));
            let manager_mutex = Arc::new(Mutex::new(current_manager));

            group.par_iter().try_for_each(|&system_id| {
                let world_lock = world_mutex.clone();
                let manager_lock = manager_mutex.clone();

                let mut world_guard = world_lock
                    .lock()
                    .map_err(|_| EcsError::SystemError("Failed to lock world".to_string()))?;
                let mut manager_guard = manager_lock.lock().map_err(|_| {
                    EcsError::SystemError("Failed to lock system manager".to_string())
                })?;

                manager_guard.run_single_system(system_id, &mut *world_guard)
            })?;

            // Get the resources back from the mutexes
            current_world = Arc::try_unwrap(world_mutex)
                .map_err(|_| EcsError::SystemError("Failed to unwrap world".to_string()))?
                .into_inner()
                .map_err(|_| EcsError::SystemError("Failed to unlock world".to_string()))?;

            current_manager = Arc::try_unwrap(manager_mutex)
                .map_err(|_| EcsError::SystemError("Failed to unwrap manager".to_string()))?
                .into_inner()
                .map_err(|_| EcsError::SystemError("Failed to unlock manager".to_string()))?;
        }

        Ok((current_world, current_manager))
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}
