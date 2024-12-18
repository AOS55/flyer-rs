use crate::resources::{AgentConfig, AgentState, SimulationMode};
use crate::systems::{apply_action, capture_frame, collect_state, ScreenshotState};

use bevy::prelude::*;

/// Plugin that manages agent interactions with the simulation
pub struct AgentPlugin {
    config: AgentConfig,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Id {
    Named(String),
    Entity(Entity),
}

#[derive(Component, Hash, Eq, PartialEq, Debug, Clone)]
pub struct Identifier {
    pub id: Id,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AgentSystemSet {
    StateCollection,
    ActionApplication,
    RenderCapture,
}

impl AgentPlugin {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }
}

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        // Add agent state resource
        app.insert_resource(AgentState::new(&self.config))
            .insert_resource(ScreenshotState::default())
            // Configure system sets for ordered execution
            .configure_sets(
                Update,
                (
                    AgentSystemSet::StateCollection,
                    AgentSystemSet::ActionApplication,
                    AgentSystemSet::RenderCapture,
                )
                    .chain(),
            )
            // Add core agent systems
            .add_systems(
                Update,
                (
                    collect_state.in_set(AgentSystemSet::StateCollection),
                    apply_action.in_set(AgentSystemSet::ActionApplication),
                )
                    .chain(),
            );

        // Add render capture system only in render mode
        if self.config.mode == SimulationMode::RGBArray {
            app.add_systems(Update, capture_frame.in_set(AgentSystemSet::RenderCapture));
        }
    }
}
