use bevy::prelude::*;

use crate::{
    components::{
        AircraftRenderState, AircraftType, Attitude, CollisionComponent, DubinsAircraftConfig,
        DubinsAircraftState, PlayerController, TaskComponent,
    },
    plugins::{AircraftBaseInitialized, AircraftPluginBase, Id, Identifier, StartupStage},
};

/// A plugin to handle Dubins aircraft behavior, rendering, and input.
/// Dubins aircraft follow simplified motion rules, suitable for lightweight physics simulations.
pub struct DubinsAircraftPlugin {
    /// Configuration for the Dubins aircraft
    configs: Vec<DubinsAircraftConfig>,
}

impl DubinsAircraftPlugin {
    /// Creates a new instance of the `DubinsAircraftPlugin` with the given configuration.
    ///
    /// # Arguments
    /// * `config` - The configuration for the Dubins aircraft, such as max speed and random start settings.
    pub fn new_single(config: DubinsAircraftConfig) -> Self {
        DubinsAircraftPlugin {
            configs: vec![config],
        }
    }

    /// Creates a new `DubinsAircraftPlugin` with a vector of configurations.
    pub fn new_vector(configs: Vec<DubinsAircraftConfig>) -> Self {
        DubinsAircraftPlugin { configs }
    }

    /// Spawns the Dubins aircraft entity during the startup phase.
    ///
    /// # Arguments
    /// * `commands` - Used to spawn the entity into the Bevy ECS.
    /// * `config` - The configuration used to initialize the Dubins aircraft.
    fn spawn_aircraft(mut commands: Commands, configs: &[DubinsAircraftConfig]) {
        for config in configs {
            commands.spawn((
                config.clone(),
                DubinsAircraftState::from_config(&config.start_config),
                PlayerController::new(),
                Name::new(config.name.to_string()), // Name of Bevy component
                CollisionComponent::default(),
                info!("Spawning Dubins aircraft: {}", config.name),
                Identifier {
                    id: Id::Named(config.name.to_string()), // Id name
                },
                AircraftRenderState {
                    attitude: Attitude::Level,
                },
                AircraftType::TwinOtter,
                TaskComponent {
                    task_type: config.task_config.clone(),
                    terminated: false,
                    weight: 1.0,
                },
            ));
        }
    }
}

impl Plugin for DubinsAircraftPlugin {
    /// Builds the `DubinsAircraftPlugin` by registering systems and resources.
    fn build(&self, app: &mut App) {
        let configs = self.configs.clone(); // Clone the config here

        // 1. Load and setup the aircraft assets (textures, layouts), if not already present.
        if !app.world().contains_resource::<AircraftBaseInitialized>() {
            app.add_systems(
                Startup,
                (AircraftPluginBase::setup_assets).in_set(StartupStage::BuildUtilities),
            );
            app.insert_resource(AircraftBaseInitialized);
        }
        // 2. Spawn the Dubins aircraft entity during the startup phase.
        app.add_systems(
            Startup,
            (move |commands: Commands| Self::spawn_aircraft(commands, &configs))
                .in_set(StartupStage::BuildAircraft), // Configure the system, not its result
        );
    }
}
