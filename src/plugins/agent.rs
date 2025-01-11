use crate::{
    resources::{AgentConfig, AgentState},
    systems::collect_state,
};

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct LatestFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl LatestFrame {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: Vec::new(),
            width,
            height,
        }
    }

    pub fn update(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}

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

impl Identifier {
    pub fn to_string(&self) -> String {
        match &self.id {
            Id::Named(name) => name.clone(),
            Id::Entity(entity) => entity.to_string(),
        }
    }
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
            .add_systems(FixedUpdate, (collect_state).chain());

        // Add render capture system only in render mode
        // if self.config.mode == SimulationMode::RGBArray {
        //     app.add_systems(Update, capture_frame.in_set(AgentSystemSet::RenderCapture));
        // }
    }
}
