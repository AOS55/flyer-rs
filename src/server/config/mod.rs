use bevy::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    components::{AircraftConfig, StartConfig, TerminalConditions},
    resources::{AgentConfig, EnvironmentConfig, PhysicsConfig, TerrainConfig, UpdateMode},
    server::{
        config::builders::{
            DubinsAircraftConfigBuilder, FullAircraftConfigBuilder, HeightNoiseConfigBuilder,
            NoiseConfigBuilder, PhysicsConfigBuilder, RandomHeadingConfigBuilder,
            RandomPosConfigBuilder, RandomSpeedConfigBuilder, RandomStartConfigBuilder,
            TerrainConfigBuilder,
        },
        ActionSpace, ObservationSpace,
    },
    utils::RngManager,
};

mod builders;
mod errors;

pub use builders::*;
pub use errors::ConfigError;

#[derive(Debug, Clone)]
pub struct EnvConfig {
    // Master Seed
    pub seed: u64,

    // Time Configuration
    pub max_episode_steps: u32,
    pub steps_per_action: usize,
    pub time_step: f64,

    // Method to update
    pub update_mode: UpdateMode,

    // Aircraft Configuration
    pub aircraft_configs: HashMap<String, AircraftConfig>,
    pub action_spaces: HashMap<String, ActionSpace>,
    pub observation_spaces: HashMap<String, ObservationSpace>,

    // Environment/physics Configuration
    pub physics_config: PhysicsConfig,
    pub environment_config: EnvironmentConfig,

    // Terrain Configuration
    pub terrain_config: TerrainConfig,

    // Agent Configuration
    pub agent_config: AgentConfig,

    // Terminal conditions
    pub terminal_conditions: TerminalConditions,
}

impl EnvConfig {
    pub fn from_json(json_str: &Value) -> Result<Self, ConfigError> {
        let builder = EnvConfigBuilder::from_json(json_str)?;
        builder.build()
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        EnvConfigBuilder::new()
            .build()
            .expect("Default configuration should always be valid")
    }
}

impl EnvConfig {
    pub fn rebuild_with_seed(&self, new_seed: u64) -> Result<Self, ConfigError> {
        info!("Rebuilding config with seed: {}", new_seed);
        let mut builder = EnvConfigBuilder::new()
            .max_episode_steps(self.max_episode_steps)
            .steps_per_action(self.steps_per_action)
            .time_step(self.time_step);

        // Set up RNG manager with new seed
        let rng_manager = RngManager::new(new_seed);
        builder.rng_manager = Some(rng_manager.clone());

        // Rebuild aircraft configurations
        for (id, config) in &self.aircraft_configs {
            match config {
                AircraftConfig::Dubins(dubins) => {
                    let mut aircraft_builder = DubinsAircraftConfigBuilder::new();
                    aircraft_builder.name = Some(dubins.name.clone());
                    aircraft_builder.max_speed = Some(dubins.max_speed);
                    aircraft_builder.min_speed = Some(dubins.min_speed);
                    aircraft_builder.acceleration = Some(dubins.acceleration);
                    aircraft_builder.max_bank_angle = Some(dubins.max_bank_angle);
                    aircraft_builder.max_turn_rate = Some(dubins.max_turn_rate);
                    aircraft_builder.max_climb_rate = Some(dubins.max_climb_rate);
                    aircraft_builder.max_descent_rate = Some(dubins.max_descent_rate);
                    aircraft_builder.seed = Some(new_seed);

                    // Handle start configuration
                    match &dubins.start_config {
                        StartConfig::Fixed(fixed) => {
                            let fixed_builder = FixedStartConfigBuilder {
                                position: Some(fixed.position),
                                speed: Some(fixed.speed),
                                heading: Some(fixed.heading),
                            };
                            aircraft_builder.start_config =
                                Some(StartConfigBuilder::Fixed(fixed_builder));
                        }
                        StartConfig::Random(random) => {
                            // Create position builder
                            let position_builder = RandomPosConfigBuilder {
                                origin: Some(random.position.origin),
                                variance: Some(random.position.variance),
                                min_altitude: Some(random.position.min_altitude),
                                max_altitude: Some(random.position.max_altitude),
                            };

                            // Create speed builder
                            let speed_builder = RandomSpeedConfigBuilder {
                                min_speed: Some(random.speed.min_speed),
                                max_speed: Some(random.speed.max_speed),
                            };

                            // Create heading builder
                            let heading_builder = RandomHeadingConfigBuilder {
                                min_heading: Some(random.heading.min_heading),
                                max_heading: Some(random.heading.max_heading),
                            };

                            // Create the complete random start builder
                            let random_start_builder = RandomStartConfigBuilder {
                                position: position_builder,
                                speed: speed_builder,
                                heading: heading_builder,
                                seed: Some(new_seed),
                            };

                            aircraft_builder.start_config =
                                Some(StartConfigBuilder::Random(random_start_builder));
                        }
                    }

                    builder
                        .aircraft_builders
                        .insert(id.clone(), AircraftBuilderEnum::Dubins(aircraft_builder));
                }

                AircraftConfig::Full(full) => {
                    let mut aircraft_builder = FullAircraftConfigBuilder::new();
                    aircraft_builder.name = Some(full.name.clone());
                    aircraft_builder.ac_type = Some(full.ac_type.clone());
                    aircraft_builder.mass = Some(full.mass.clone());
                    aircraft_builder.geometry = Some(full.geometry.clone());
                    aircraft_builder.aero_coef = Some(full.aero_coef.clone());

                    // Handle start configuration
                    match &full.start_config {
                        StartConfig::Fixed(fixed) => {
                            let fixed_builder = FixedStartConfigBuilder {
                                position: Some(fixed.position),
                                speed: Some(fixed.speed),
                                heading: Some(fixed.heading),
                            };
                            aircraft_builder.start_config =
                                Some(StartConfigBuilder::Fixed(fixed_builder));
                        }
                        StartConfig::Random(random) => {
                            // Create position builder
                            let position_builder = RandomPosConfigBuilder {
                                origin: Some(random.position.origin),
                                variance: Some(random.position.variance),
                                min_altitude: Some(random.position.min_altitude),
                                max_altitude: Some(random.position.max_altitude),
                            };

                            // Create speed builder
                            let speed_builder = RandomSpeedConfigBuilder {
                                min_speed: Some(random.speed.min_speed),
                                max_speed: Some(random.speed.max_speed),
                            };

                            // Create heading builder
                            let heading_builder = RandomHeadingConfigBuilder {
                                min_heading: Some(random.heading.min_heading),
                                max_heading: Some(random.heading.max_heading),
                            };

                            // Create the complete random start builder
                            let random_start_builder = RandomStartConfigBuilder {
                                position: position_builder,
                                speed: speed_builder,
                                heading: heading_builder,
                                seed: Some(new_seed),
                            };

                            aircraft_builder.start_config =
                                Some(StartConfigBuilder::Random(random_start_builder));
                        }
                    }

                    builder
                        .aircraft_builders
                        .insert(id.clone(), AircraftBuilderEnum::Full(aircraft_builder));
                }
            }

            // Preserve action and observation spaces
            builder.action_builders.insert(
                id.clone(),
                ActionSpaceBuilder::new().act_space(self.action_spaces.get(id).unwrap().clone()),
            );
            builder.observation_builders.insert(
                id.clone(),
                ObservationSpaceBuilder::new()
                    .obs_space(self.observation_spaces.get(id).unwrap().clone()),
            );
        }

        // Rebuild physics config
        builder.physics_builder = PhysicsConfigBuilder::new()
            .max_velocity(self.physics_config.max_velocity)
            .max_angular_velocity(self.physics_config.max_angular_velocity)
            .timestep(self.physics_config.timestep);

        // Rebuild terrain config with new seed
        let mut terrain_builder = TerrainConfigBuilder::new();
        terrain_builder.seed = new_seed;

        // Copy noise configuration
        let height_noise = HeightNoiseConfigBuilder::new()
            .scale(self.terrain_config.noise.height.scale)
            .octaves(self.terrain_config.noise.height.octaves)
            .persistence(self.terrain_config.noise.height.persistence)
            .lacunarity(self.terrain_config.noise.height.lacunarity);

        // Add all noise layers
        let mut final_height_builder = height_noise;
        for layer in &self.terrain_config.noise.height.layers {
            final_height_builder = final_height_builder.add_layer(layer.clone());
        }

        let noise_builder = NoiseConfigBuilder::new().height_noise(final_height_builder);

        builder = builder.terrain_config(terrain_builder.noise_config(noise_builder));

        // Build the final config
        builder.build()
    }
}
