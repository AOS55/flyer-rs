use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    components::{
        AirData, AircraftControlSurfaces, CollisionComponent, FullAircraftConfig, PhysicsComponent,
        PlayerController, PropulsionState, SpatialComponent, TaskComponent,
        TrimCondition, 
        TrimRequest,
    },
    plugins::{Id, Identifier, StartupStage},
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
    /// If no explicit trim_condition is provided in the config,
    /// it sends a TrimRequest based on the start_state speed.
    fn spawn_aircraft(
        mut commands: Commands,
        configs: &[FullAircraftConfig],
        mut trim_requests: EventWriter<TrimRequest>, 
    ) {
        for config in configs {
            info!("Spawning full aircraft entity for '{}'...", &config.name);
            let start_state = StartState::from_config(config);

            // Spawn the entity and store its ID
            let entity = commands.spawn((
                config.clone(), // add the config as a component on the entity
                PlayerController::new(),
                AirData::default(),
                AircraftControlSurfaces::default(), // Start with default controls
                SpatialComponent::new(
                    start_state.position,
                    start_state.velocity, 
                    start_state.attitude,
                    Vector3::zeros(), // Provide default angular velocity
                ),
                CollisionComponent::from_geometry(&config.geometry),
                PhysicsComponent::new(config.mass.mass, config.mass.inertia),
                PropulsionState::new(config.propulsion.engines.len()),
                Name::new(config.name.to_string()),
                Identifier {
                    id: Id::Named(config.name.to_string()),
                },
                TaskComponent {
                    task_type: config.task_config.clone(),
                    terminated: false,
                    weight: 1.0,
                },
            )).id(); // Get the entity ID

            if config.trim_condition.is_none() {
                info!("No explicit trim condition provided for '{}'. Requesting default trim.", config.name);

                let initial_speed = start_state.speed; 

                if initial_speed > 1e-6 {
                    trim_requests.send(TrimRequest {
                        entity: entity,
                        condition: TrimCondition::StraightAndLevel {
                            airspeed: initial_speed,
                        },
                    });
                    info!("Sent TrimRequest (StraightAndLevel @ {:.2} m/s) for entity {:?}.", initial_speed, entity);
                } else {
                    warn!("Initial speed for '{}' is near zero ({:.2} m/s). Skipping automatic trim request.", config.name, initial_speed);
                }
            } else {
                info!("Using explicit trim condition provided in config for '{}'.", config.name);
            }
        }
    }
}

impl Plugin for FullAircraftPlugin {
    /// Builds the `FullAircraftPlugin` by registering systems, resources, and startup logic.
    fn build(&self, app: &mut App) {
        let configs = self.configs.clone();

        app.add_systems(
            Startup,
            (move |commands: Commands, trim_requests: EventWriter<TrimRequest>| { 
                Self::spawn_aircraft(commands, &configs, trim_requests) 
            })
            .in_set(StartupStage::BuildAircraft),
        );
    }
}

struct StartState {
    position: Vector3<f64>,
    speed: f64, 
    heading: f64, 
    velocity: Vector3<f64>, 
    attitude: UnitQuaternion<f64>,
}

impl StartState {
    /// Creates a StartState from the FullAircraftConfig's start_config field.
    fn from_config(config: &FullAircraftConfig) -> Self {
        // Determine position, speed, and heading based on the StartConfig variant
        let (position, speed, heading_rad) = match &config.start_config {
            // If Fixed, use the values directly
            crate::components::StartConfig::Fixed(fixed_config) => {
                info!("Using fixed start config: pos={:?}, speed={}, heading_rad={}", fixed_config.position, fixed_config.speed, fixed_config.heading);
                (fixed_config.position, fixed_config.speed, fixed_config.heading)
            }
            // If Random, call the generate method
            crate::components::StartConfig::Random(random_config) => {
                info!("Generating random start config...");
                let (pos, spd, head_rad) = random_config.generate();
                info!("Generated start config: pos={:?}, speed={}, heading_rad={}", pos, spd, head_rad);
                (pos, spd, head_rad)
            }
        };

        // Calculate initial velocity and attitude based on speed and heading
        // Assumes heading_rad = 0 is North, positive rotation is clockwise (typical aviation convention for heading)
        // Bevy/nalgebra use Y-up, right-handed coordinate system. Positive rotation around Y is counter-clockwise.
        // Velocity calculation: North = +Z, East = +X
        let velocity = Vector3::new(speed * heading_rad.sin(), 0.0, speed * heading_rad.cos());
        // Attitude calculation: Apply negative heading for clockwise rotation around Y-axis
        let attitude = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -heading_rad);

        Self {
            position,
            speed,
            heading: heading_rad, 
            velocity,
            attitude,
        }
    }
}
