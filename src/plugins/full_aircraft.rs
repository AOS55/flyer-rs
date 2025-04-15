use bevy::prelude::*;
use nalgebra::Vector3;

use crate::{
    components::{
        AirData, AircraftControlSurfaces, CollisionComponent, FullAircraftConfig, PhysicsComponent,
        PlayerController, PropulsionState, SpatialComponent, StartConfig, TaskComponent,
    },
    plugins::{AircraftBaseInitialized, Id, Identifier, StartupStage},
};

/// Plugin to handle "Full Aircraft" systems.
/// This plugin provides detailed physics simulation, including:
/// - Air data calculations.
/// - Aerodynamic forces and moments.
/// - Force integration.
/// - Full spatial updates for position and attitude.
pub struct FullAircraftPlugin {
    /// Configuration for the full aircraft, containing mass, geometry, and aerodynamic coefficients.
    configs: Vec<FullAircraftConfig>,
}

impl FullAircraftPlugin {
    /// Creates a new instance of the `FullAircraftPlugin` with the given configuration.
    ///
    /// # Arguments:
    /// * `config` - The configuration defining the aircraft's physical parameters.
    pub fn new_single(config: FullAircraftConfig) -> Self {
        FullAircraftPlugin {
            configs: vec![config],
        }
    }

    pub fn new_vector(configs: Vec<FullAircraftConfig>) -> Self {
        FullAircraftPlugin { configs }
    }

    /// Spawns a full aircraft entity with the required components.
    ///
    /// # Arguments:
    /// * `commands` - Used to spawn the entity into the ECS.
    /// * `config` - The full aircraft configuration, cloned for the new entity.
    fn spawn_aircraft(mut commands: Commands, configs: &[FullAircraftConfig]) {
        for config in configs {
            info!("Spawning full aircraft entity...");
            let start_state = StartState::from_config(&config);

            commands.spawn((
                config.clone(), // add the config as a resource into the entity
                PlayerController::new(),
                AirData::default(),
                AircraftControlSurfaces::default(),
                SpatialComponent::at_position_and_airspeed(
                    start_state.position,
                    start_state.speed,
                    start_state.heading,
                ),
                CollisionComponent::from_geometry(&config.geometry),
                PhysicsComponent::new(config.mass.mass, config.mass.inertia),
                PropulsionState::new(config.propulsion.engines.len()), // Hardcoded to 2 engines for now
                Name::new(config.name.to_string()),
                Identifier {
                    id: Id::Named(config.name.to_string()), // Id name
                },
                TaskComponent {
                    task_type: config.task_config.clone(),
                    terminated: false,
                    weight: 1.0,
                },
            ));
        }
    }
}

impl Plugin for FullAircraftPlugin {
    /// Builds the `FullAircraftPlugin` by registering systems, resources, and startup logic.
    fn build(&self, app: &mut App) {
        let configs = self.configs.clone();

        // 1. Setup aircraft assets (textures, sprite layouts)
        if !app.world().contains_resource::<AircraftBaseInitialized>() {
            // app.add_systems(
            //     Startup,
            //     (AircraftPluginBase::setup_assets,).in_set(StartupStage::BuildUtilities),
            // );
        }
        // 3. Spawn the full aircraft entity during startup
        app.add_systems(
            Startup,
            (move |commands: Commands| Self::spawn_aircraft(commands, &configs))
                .in_set(StartupStage::BuildAircraft),
        );
    }
}

struct StartState {
    position: Vector3<f64>,
    speed: f64,
    heading: f64,
}

impl StartState {
    fn from_config(config: &FullAircraftConfig) -> Self {
        match &config.start_config {
            StartConfig::Fixed(fixed_config) => Self {
                position: fixed_config.position,
                speed: fixed_config.speed,
                heading: fixed_config.heading,
            },
            StartConfig::Random(random_config) => {
                let (position, speed, heading) = random_config.generate();
                Self {
                    position,
                    speed,
                    heading,
                }
            }
        }
    }
}
