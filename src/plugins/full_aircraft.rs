use bevy::prelude::*;
use nalgebra::Vector3;

use crate::{
    components::{
        AirData, AircraftControlSurfaces, CollisionComponent, FullAircraftConfig, PhysicsComponent,
        PlayerController, PropulsionState, SpatialComponent, StartConfig, TaskComponent,
    },
    plugins::{AircraftPluginBase, Id, Identifier, StartupStage},
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
            PropulsionState::new(2), // Hardcoded to 2 engines for now
            Name::new(config.name.to_string()),
            Identifier {
                id: Id::Named(config.name.to_string()), // Id name
            },
            TaskComponent {
                task_type: config.task_config,
                terminated: false,
                weight: 1.0,
            },
        ));
    }
}

impl Plugin for FullAircraftPlugin {
    /// Builds the `FullAircraftPlugin` by registering systems, resources, and startup logic.
    fn build(&self, app: &mut App) {
        let config = self.config.clone();

        // 1. Setup aircraft assets (textures, sprite layouts)
        app.add_systems(
            Startup,
            (AircraftPluginBase::setup_assets,).in_set(StartupStage::BuildUtilities),
        )
        // 2. Configure the physics update sets:
        // AirData -> Aerodynamics -> Forces -> Integration
        // .configure_sets(
        //     FixedUpdate,
        //     (
        //         ComplexPhysicsSet::AirData,
        //         ComplexPhysicsSet::Aerodynamics,
        //         ComplexPhysicsSet::Forces,
        //         ComplexPhysicsSet::Integration,
        //     )
        //         .chain(),
        // )
        // 3. Spawn the full aircraft entity during startup
        .add_systems(
            Startup,
            (move |commands: Commands| Self::setup_aircraft(commands, config.clone()))
                .in_set(StartupStage::BuildAircraft),
        );
        // 4. Add systems to handle full aircraft physics and integration
        // .add_systems(
        //     FixedUpdate,
        //     (
        //         air_data_system.in_set(ComplexPhysicsSet::AirData),
        //         aero_force_system.in_set(ComplexPhysicsSet::Aerodynamics),
        //         force_calculator_system.in_set(ComplexPhysicsSet::Forces),
        //         physics_integrator_system.in_set(ComplexPhysicsSet::Integration),
        //     ),
        // );

        // 5. Set the fixed timestep resource for physics calculations
        // app.init_resource::<Time<Fixed>>()
        //     .insert_resource(Time::<Fixed>::from_seconds(1.0 / 120.0));
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
