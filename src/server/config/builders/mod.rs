use rand;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

mod act;
mod aircraft;
mod environment;
mod obs;
mod physics;
mod start;
mod task;
mod terrain;

use crate::{
    components::AircraftConfig,
    resources::{AgentConfig, RenderMode, UpdateMode},
    server::config::errors::ConfigError,
    server::{ActionSpace, EnvConfig, ObservationSpace},
    utils::RngManager,
};

pub use act::ActionSpaceBuilder;
pub use aircraft::{
    create_aircraft_builder, AircraftBuilder, AircraftBuilderEnum, DubinsAircraftConfigBuilder,
    FullAircraftConfigBuilder,
};
use environment::EnvironmentConfigBuilder;
pub use obs::ObservationSpaceBuilder;
pub use physics::PhysicsConfigBuilder;
pub use start::{
    FixedStartConfigBuilder, RandomHeadingConfigBuilder, RandomPosConfigBuilder,
    RandomSpeedConfigBuilder, RandomStartConfigBuilder, StartConfigBuilder,
};
pub use task::TaskConfigBuilder;
pub use terrain::{HeightNoiseConfigBuilder, NoiseConfigBuilder, TerrainConfigBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConfigBuilder {
    #[serde(skip)]
    pub rng_manager: Option<RngManager>,
    pub max_episode_steps: Option<u32>,
    pub steps_per_action: Option<usize>,
    pub time_step: Option<f64>,
    #[serde(skip)]
    pub aircraft_builders: HashMap<String, AircraftBuilderEnum>,
    #[serde(skip)]
    pub action_builders: HashMap<String, ActionSpaceBuilder>,
    #[serde(skip)]
    pub observation_builders: HashMap<String, ObservationSpaceBuilder>,
    #[serde(skip)]
    pub physics_builder: PhysicsConfigBuilder,
    environment_builder: EnvironmentConfigBuilder,
    #[serde(skip)]
    pub terrain_builder: TerrainConfigBuilder,
    pub agent_config: Option<AgentConfig>,
}

impl Default for EnvConfigBuilder {
    fn default() -> Self {
        Self {
            rng_manager: None,
            max_episode_steps: None,
            steps_per_action: None,
            time_step: None,
            aircraft_builders: HashMap::new(),
            action_builders: HashMap::new(),
            observation_builders: HashMap::new(),
            physics_builder: PhysicsConfigBuilder::default(),
            environment_builder: EnvironmentConfigBuilder::default(),
            terrain_builder: TerrainConfigBuilder::default(),
            agent_config: None,
        }
    }
}

impl EnvConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_episode_steps(mut self, steps: u32) -> Self {
        self.max_episode_steps = Some(steps);
        self
    }

    pub fn steps_per_action(mut self, steps: usize) -> Self {
        self.steps_per_action = Some(steps);
        self
    }

    pub fn time_step(mut self, dt: f64) -> Self {
        self.time_step = Some(dt);
        self
    }

    pub fn terrain_config(mut self, builder: TerrainConfigBuilder) -> Self {
        self.terrain_builder = builder;
        self
    }

    pub fn from_json(json_value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        // Parse seed
        let seed = json_value
            .get("seed")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(rand::random);

        let rng_manager = RngManager::new(seed);
        builder.rng_manager = Some(rng_manager.clone());

        // Parse basic configuration
        if let Some(steps) = json_value.get("max_episode_steps").and_then(|v| v.as_u64()) {
            builder = builder.max_episode_steps(steps as u32);
        }
        if let Some(steps) = json_value.get("steps_per_action").and_then(|v| v.as_u64()) {
            builder = builder.steps_per_action(steps as usize);
        }
        if let Some(dt) = json_value.get("time_step").and_then(|v| v.as_f64()) {
            builder = builder.time_step(dt);
        }

        // Parse aircraft configurations
        if let Some(aircraft_configs) = json_value.get("aircraft_config").and_then(|v| v.as_array())
        {
            for (i, config) in aircraft_configs.iter().enumerate() {
                let id = format!("aircraft_{}", i);
                let mut aircraft_agent = create_aircraft_builder(config, seed)?; // Might need to add to the seed to seperate aircraft or rng to the seed

                // Set id name in builder
                match &mut aircraft_agent.aircraft_builder {
                    AircraftBuilderEnum::Dubins(builder) => builder.name = Some(id.clone()),
                    AircraftBuilderEnum::Full(builder) => builder.name = Some(id.clone()),
                }

                // Initialize each aircraft with its own RNG stream
                builder
                    .aircraft_builders
                    .insert(id.clone(), aircraft_agent.aircraft_builder);

                builder
                    .action_builders
                    .insert(id.clone(), aircraft_agent.action_builder);

                builder
                    .observation_builders
                    .insert(id.clone(), aircraft_agent.observation_builder);
            }
        }

        // Parse terrain configuration
        if let Some(terrain_config) = json_value.get("terrain_config") {
            let mut config = TerrainConfigBuilder::from_json(terrain_config)?;
            config.seed = seed;
            builder = builder.terrain_config(config);
        }

        if let Some(physics_config) = json_value.get("physics_config") {
            builder.physics_builder = PhysicsConfigBuilder::from_json(physics_config)?;
        }

        if let Some(environment_config) = json_value.get("environment_config") {
            builder.environment_builder = EnvironmentConfigBuilder::from_json(environment_config)?;
        }

        // Parse agent config
        if let Some(agent_config) = json_value.get("agent_config") {
            let mode = match agent_config.get("mode").and_then(|v| v.as_str()) {
                Some("human") => RenderMode::Human,
                Some("RGBArray") => RenderMode::RGBArray,
                _ => RenderMode::Human, // Default
            };

            let width = agent_config
                .get("render_width")
                .and_then(|v| v.as_f64())
                .unwrap_or(800.0);
            let height = agent_config
                .get("render_height")
                .and_then(|v| v.as_f64())
                .unwrap_or(600.0);

            builder.agent_config = Some(AgentConfig {
                mode,
                render_width: width as f32,
                render_height: height as f32,
                ..AgentConfig::default()
            });
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<EnvConfig, ConfigError> {
        let rng_manager = self
            .rng_manager
            .unwrap_or_else(|| RngManager::new(rand::random()));

        // Build configurations from HashMaps
        let mut aircraft_configs: HashMap<String, AircraftConfig> = HashMap::new();
        let mut action_spaces: HashMap<String, ActionSpace> = HashMap::new();
        let mut observation_spaces: HashMap<String, ObservationSpace> = HashMap::new();

        // Process all builders
        for (id, builder) in self.aircraft_builders {
            aircraft_configs.insert(id.clone(), builder.build()?);
        }

        for (id, builder) in self.action_builders {
            action_spaces.insert(id.clone(), builder.build()?);
        }

        for (id, builder) in self.observation_builders {
            observation_spaces.insert(id.clone(), builder.build()?);
        }

        Ok(EnvConfig {
            seed: rng_manager.master_seed(),
            update_mode: UpdateMode::Gym,
            max_episode_steps: self.max_episode_steps.unwrap_or(1000),
            steps_per_action: self.steps_per_action.unwrap_or(10),
            time_step: self.time_step.unwrap_or(1.0 / 60.0),
            aircraft_configs,
            action_spaces,
            observation_spaces,
            physics_config: self.physics_builder.build()?,
            environment_config: self.environment_builder.build()?,
            terrain_config: self.terrain_builder.build()?,
            agent_config: self.agent_config.unwrap_or_default(),
        })
    }
}
