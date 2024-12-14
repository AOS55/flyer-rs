use bevy::prelude::*;

use crate::components::{AircraftState, FullAircraftConfig, SpatialComponent};
use crate::plugins::{AircraftPluginBase, ComplexPhysicsSet, StartupSet};
use crate::systems::{
    aero_force_system, air_data_system, force_calculator_system, physics_integrator_system,
};

pub struct FullAircraftPlugin {
    config: FullAircraftConfig,
}

impl FullAircraftPlugin {
    pub fn new(config: FullAircraftConfig) -> Self {
        FullAircraftPlugin { config }
    }

    fn setup_aircraft(mut commands: Commands, config: FullAircraftConfig) {
        commands.spawn((
            config.clone(),
            AircraftState::default(),
            SpatialComponent::default(),
            Name::new(config.name.to_string()),
        ));
    }
}

impl Plugin for FullAircraftPlugin {
    fn build(&self, app: &mut App) {
        let config = self.config.clone();

        app.add_systems(Startup, (AircraftPluginBase::setup_assets,).chain())
            .configure_sets(
                FixedUpdate,
                (
                    ComplexPhysicsSet::AirData,
                    ComplexPhysicsSet::Aerodynamics,
                    ComplexPhysicsSet::Forces,
                    ComplexPhysicsSet::Integration,
                )
                    .chain(),
            )
            .add_systems(
                Startup,
                (move |commands: Commands| Self::setup_aircraft(commands, config.clone()))
                    .in_set(StartupSet::SpawnPlayer),
            )
            .add_systems(
                FixedUpdate,
                (
                    air_data_system.in_set(ComplexPhysicsSet::AirData),
                    aero_force_system.in_set(ComplexPhysicsSet::Aerodynamics),
                    force_calculator_system.in_set(ComplexPhysicsSet::Forces),
                    physics_integrator_system.in_set(ComplexPhysicsSet::Integration),
                ),
            );

        app.init_resource::<Time<Fixed>>()
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 120.0));
    }
}
