use bevy::prelude::*;

use crate::{
    components::{
        AircraftRenderState, AircraftType, Attitude, DubinsAircraftConfig, DubinsAircraftState,
        PlayerController,
    },
    plugins::{AircraftPluginBase, Id, Identifier, StartupStage},
    // resources::step_condition,
    // systems::{aircraft_render_system, dubins_aircraft_system, dubins_keyboard_system},
};

/// A plugin to handle Dubins aircraft behavior, rendering, and input.
/// Dubins aircraft follow simplified motion rules, suitable for lightweight physics simulations.
pub struct DubinsAircraftPlugin {
    /// Configuration for the Dubins aircraft
    config: DubinsAircraftConfig,
}

impl DubinsAircraftPlugin {
    /// Creates a new instance of the `DubinsAircraftPlugin` with the given configuration.
    ///
    /// # Arguments
    /// * `config` - The configuration for the Dubins aircraft, such as max speed and random start settings.
    pub fn new(config: DubinsAircraftConfig) -> Self {
        DubinsAircraftPlugin { config }
    }

    /// Spawns the Dubins aircraft entity during the startup phase.
    ///
    /// # Arguments
    /// * `commands` - Used to spawn the entity into the Bevy ECS.
    /// * `config` - The configuration used to initialize the Dubins aircraft.
    fn setup_aircraft(mut commands: Commands, config: DubinsAircraftConfig) {
        commands.spawn((
            config.clone(),
            DubinsAircraftState::from_config(&config.start_config),
            PlayerController::new(),
            Name::new(config.name.to_string()), // Name of Bevy component
            info!("Spawning Dubins aircraft: {}", config.name),
            Identifier {
                id: Id::Named(config.name.to_string()), // Id name
            },
            AircraftRenderState {
                attitude: Attitude::Level,
            },
            AircraftType::TwinOtter,
        ));
    }
}

impl Plugin for DubinsAircraftPlugin {
    /// Builds the `DubinsAircraftPlugin` by registering systems and resources.
    fn build(&self, app: &mut App) {
        let config = self.config.clone(); // Clone the config here

        // 1. Load and setup the aircraft assets (textures, layouts).
        app.add_systems(
            Startup,
            (AircraftPluginBase::setup_assets).in_set(StartupStage::BuildUtilities),
        )
        // 2. Spawn the Dubins aircraft entity during the startup phase.
        .add_systems(
            Startup,
            (move |commands: Commands| Self::setup_aircraft(commands, config.clone()))
                .in_set(StartupStage::BuildAircraft), // Configure the system, not its result
        );

        // Done in the main app loop now
        // // 3. Add systems to handle input, update physics, and render the Dubins aircraft.
        // .add_systems(
        //     FixedUpdate,
        //     (dubins_aircraft_system, aircraft_render_system)
        //         .chain()
        //         .run_if(step_condition),
        // );

        // // 5. Initialize the fixed timestep resource for the physics simulation.
        // app.init_resource::<Time<Fixed>>()
        //     .insert_resource(Time::<Fixed>::from_seconds(1.0 / 120.0));
    }
}
