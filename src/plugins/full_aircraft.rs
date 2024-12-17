use bevy::prelude::*;

use crate::components::{AircraftState, FullAircraftConfig, SpatialComponent};
use crate::plugins::{AircraftPluginBase, ComplexPhysicsSet, StartupSet};
use crate::systems::{
    aero_force_system, air_data_system, force_calculator_system, physics_integrator_system,
};

/// Plugin to handle "Full Aircraft" systems.
/// This plugin provides detailed physics simulation, including:
/// - Air data calculations.
/// - Aerodynamic forces and moments.
/// - Force integration.
/// - Full spatial updates for position and attitude.
pub struct FullAircraftPlugin {
    /// Configuration for the full aircraft, containing mass, geometry, and aerodynamic coefficients.
    config: FullAircraftConfig,
}

impl FullAircraftPlugin {
    /// Creates a new instance of the `FullAircraftPlugin` with the given configuration.
    ///
    /// # Arguments:
    /// * `config` - The configuration defining the aircraft's physical parameters.
    pub fn new(config: FullAircraftConfig) -> Self {
        FullAircraftPlugin { config }
    }

    /// Spawns a full aircraft entity with the required components.
    ///
    /// # Arguments:
    /// * `commands` - Used to spawn the entity into the ECS.
    /// * `config` - The full aircraft configuration, cloned for the new entity.
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
    /// Builds the `FullAircraftPlugin` by registering systems, resources, and startup logic.
    fn build(&self, app: &mut App) {
        let config = self.config.clone();

        // 1. Setup aircraft assets (textures, sprite layouts)
        app.add_systems(Startup, (AircraftPluginBase::setup_assets,).chain())
            // 2. Configure the physics update sets:
            // AirData -> Aerodynamics -> Forces -> Integration
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
            // 3. Spawn the full aircraft entity during startup
            .add_systems(
                Startup,
                (move |commands: Commands| Self::setup_aircraft(commands, config.clone()))
                    .in_set(StartupSet::SpawnPlayer),
            )
            // 4. Add systems to handle full aircraft physics and integration
            .add_systems(
                FixedUpdate,
                (
                    air_data_system.in_set(ComplexPhysicsSet::AirData),
                    aero_force_system.in_set(ComplexPhysicsSet::Aerodynamics),
                    force_calculator_system.in_set(ComplexPhysicsSet::Forces),
                    physics_integrator_system.in_set(ComplexPhysicsSet::Integration),
                ),
            );

        // 5. Set the fixed timestep resource for physics calculations
        app.init_resource::<Time<Fixed>>()
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 120.0));
    }
}
