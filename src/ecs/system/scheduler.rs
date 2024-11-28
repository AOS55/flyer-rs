use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::{HashMap, HashSet};

use super::SystemId;
use crate::ecs::error::Result;
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
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            dependency_graph: Graph::new(),
            node_indices: HashMap::new(),
        }
    }

    pub fn add_system(&mut self, id: SystemId, stage: Stage, dependencies: Vec<SystemId>) {
        self.systems.insert(id, stage);
        let node_idx = self.dependency_graph.add_node(id);
        self.node_indices.insert(id, node_idx);

        for dep_id in dependencies {
            if let Some(&dep_idx) = self.node_indices.get(&dep_id) {
                self.dependency_graph.add_edge(dep_idx, node_idx, ());
            }
        }
    }

    pub fn remove_system(&mut self, id: SystemId) {
        self.systems.remove(&id);
        if let Some(idx) = self.node_indices.remove(&id) {
            self.dependency_graph.remove_node(idx);
        }
    }

    pub fn build_execution_order(&self) -> Vec<Vec<SystemId>> {
        let mut stages = HashMap::new();
        for (id, &stage) in &self.systems {
            stages.entry(stage).or_insert_with(HashSet::new).insert(*id);
        }

        let mut sorted_systems = Vec::new();
        let system_order = toposort(&self.dependency_graph, None).unwrap_or_default();

        for &stage in &[Stage::PreUpdate, Stage::Update, Stage::PostUpdate] {
            let stage_systems: Vec<SystemId> = system_order
                .iter()
                .copied()
                .filter(|node_idx| {
                    let system_id = self.dependency_graph[*node_idx];
                    stages.get(&stage).map_or(false, |s| s.contains(&system_id))
                })
                .map(|node_idx| self.dependency_graph[node_idx])
                .collect();

            if !stage_systems.is_empty() {
                sorted_systems.push(stage_systems);
            }
        }

        sorted_systems
    }

    pub fn execute_systems(
        &self,
        world: &mut World,
        manager: &mut super::SystemManager,
    ) -> Result<()> {
        let execution_order = self.build_execution_order();

        for stage_systems in execution_order {
            for system_id in stage_systems {
                manager.run_system(system_id, world)?;
            }
        }
        Ok(())
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}
