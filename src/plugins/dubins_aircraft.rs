use bevy::prelude::*;

use crate::components::{
    AircraftRenderState, AircraftType, Attitude, DubinsAircraftConfig, DubinsAircraftState,
    PlayerController,
};
use crate::plugins::{AircraftPluginBase, SimplePhysicsSet, StartupSet};
use crate::systems::{aircraft_render_system, dubins_aircraft_system, dubins_keyboard_system};

pub struct DubinsAircraftPlugin {
    config: DubinsAircraftConfig,
}

impl DubinsAircraftPlugin {
    pub fn new(config: DubinsAircraftConfig) -> Self {
        DubinsAircraftPlugin { config }
    }

    fn setup_aircraft(mut commands: Commands, config: DubinsAircraftConfig) {
        commands.spawn((
            config.clone(),
            DubinsAircraftState::random_position(config.random_start_config),
            PlayerController::new(),
            Name::new(config.name.to_string()),
            AircraftRenderState {
                attitude: Attitude::Level,
            },
            AircraftType::TwinOtter,
        ));
    }
}

impl Plugin for DubinsAircraftPlugin {
    fn build(&self, app: &mut App) {
        let config = self.config.clone(); // Clone the config here

        app.add_systems(Startup, (AircraftPluginBase::setup_assets,).chain())
            .configure_sets(
                FixedUpdate,
                (SimplePhysicsSet::Input, SimplePhysicsSet::Update).chain(),
            )
            .add_systems(
                Startup,
                (move |commands: Commands| Self::setup_aircraft(commands, config.clone()))
                    .in_set(StartupSet::SpawnPlayer), // Configure the system, not its result
            )
            .add_systems(
                FixedUpdate,
                (
                    dubins_keyboard_system.in_set(SimplePhysicsSet::Input),
                    dubins_aircraft_system.in_set(SimplePhysicsSet::Update),
                    aircraft_render_system,
                ),
            );

        app.init_resource::<Time<Fixed>>()
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 120.0));
    }
}
