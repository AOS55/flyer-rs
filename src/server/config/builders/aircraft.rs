use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    components::{
        AircraftAeroCoefficients, AircraftConfig, AircraftGeometry, AircraftType,
        DubinsAircraftConfig, FullAircraftConfig, MassModel, PowerplantConfig, PropulsionConfig,
        StartConfig, TaskType, TrimCondition,
    },
    server::{
        config::{
            builders::{
                ActionSpaceBuilder, FixedStartConfigBuilder, ObservationSpaceBuilder,
                RandomStartConfigBuilder, StartConfigBuilder, TaskConfigBuilder,
            },
            errors::ConfigError,
        },
        obs::ContinuousObservationSpace,
        ActionSpace, ObservationSpace,
    },
};

/// Configuration for trim conditions that can be applied to aircraft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimConditionConfig {
    /// Airspeed for the trim condition in m/s
    pub airspeed: f64,
    /// Type of trim condition
    pub condition_type: String,
    /// Climb angle in radians (for steady climb condition)
    pub gamma: Option<f64>,
    /// Bank angle in radians (for coordinated turn condition)
    pub bank_angle: Option<f64>,
}

impl Default for TrimConditionConfig {
    fn default() -> Self {
        Self {
            airspeed: 70.0, // Default to 70 m/s cruising airspeed
            condition_type: "StraightAndLevel".to_string(),
            gamma: None,
            bank_angle: None,
        }
    }
}

impl TrimConditionConfig {
    pub fn to_trim_condition(&self) -> TrimCondition {
        match self.condition_type.as_str() {
            "SteadyClimb" => {
                let gamma = self.gamma.unwrap_or(0.05); // Default ~3 degrees climb
                TrimCondition::SteadyClimb { 
                    airspeed: self.airspeed,
                    gamma 
                }
            }
            "CoordinatedTurn" => {
                let bank_angle = self.bank_angle.unwrap_or(0.26); // Default ~15 degrees bank
                TrimCondition::CoordinatedTurn { 
                    airspeed: self.airspeed,
                    bank_angle 
                }
            }
            _ => {
                // Default to straight and level if not specified or unknown
                TrimCondition::StraightAndLevel { airspeed: self.airspeed }
            }
        }
    }
}

pub struct AircraftAgentBuilder {
    pub aircraft_builder: AircraftBuilderEnum,
    pub observation_builder: ObservationSpaceBuilder,
    pub action_builder: ActionSpaceBuilder,
}

// Simplified to just one trait for building aircraft
pub trait AircraftBuilder {
    fn build(&self) -> Result<AircraftConfig, ConfigError>;
}

#[derive(Debug, Clone)]
pub enum AircraftBuilderEnum {
    Dubins(DubinsAircraftConfigBuilder),
    Full(FullAircraftConfigBuilder),
}

impl AircraftBuilder for AircraftBuilderEnum {
    fn build(&self) -> Result<AircraftConfig, ConfigError> {
        match self {
            AircraftBuilderEnum::Dubins(builder) => builder.build(),
            AircraftBuilderEnum::Full(builder) => builder.build(),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DubinsAircraftConfigBuilder {
    pub name: Option<String>,
    pub max_speed: Option<f64>,
    pub min_speed: Option<f64>,
    pub acceleration: Option<f64>,
    pub max_bank_angle: Option<f64>,
    pub max_turn_rate: Option<f64>,
    pub max_climb_rate: Option<f64>,
    pub max_descent_rate: Option<f64>,
    pub start_config: Option<StartConfigBuilder>,
    pub task_config: Option<TaskConfigBuilder>,
    pub seed: Option<u64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FullAircraftConfigBuilder {
    pub name: Option<String>,
    pub ac_type: Option<AircraftType>,
    pub mass: Option<MassModel>,
    pub geometry: Option<AircraftGeometry>,
    pub aero_coef: Option<AircraftAeroCoefficients>,
    pub propulsion_config: Option<PropulsionConfig>,
    pub start_config: Option<StartConfigBuilder>,
    pub task_config: Option<TaskConfigBuilder>,
    pub seed: Option<u64>,
    /// Optional trim condition to apply during initialization
    pub trim_condition: Option<TrimConditionConfig>,
}

impl DubinsAircraftConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(value: &Value, seed: u64) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        builder.name = value.get("name").and_then(|v| v.as_str()).map(String::from);
        builder.seed = Some(seed);

        if let Some(config) = value.get("config") {
            builder.max_speed = config.get("max_speed").and_then(|v| v.as_f64());
            builder.min_speed = config.get("min_speed").and_then(|v| v.as_f64());
            builder.acceleration = config.get("acceleration").and_then(|v| v.as_f64());
            builder.max_bank_angle = config.get("max_bank_angle").and_then(|v| v.as_f64());
            builder.max_turn_rate = config.get("max_turn_rate").and_then(|v| v.as_f64());
            builder.max_climb_rate = config.get("max_climb_rate").and_then(|v| v.as_f64());
            builder.max_descent_rate = config.get("max_descent_rate").and_then(|v| v.as_f64());
        }

        // Create RandomStartConfigBuilder with either provided config or defaults
        if let Some(start_config) = value.get("start_config") {
            let config_type = start_config
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ConfigError::MissingRequired("start_config.type".into()))?;

            builder.start_config = Some(match config_type {
                "fixed" => {
                    if let Some(config) = start_config.get("config") {
                        StartConfigBuilder::Fixed(FixedStartConfigBuilder::from_json(config)?)
                    } else {
                        return Err(ConfigError::MissingRequired("start_config.config".into()));
                    }
                }
                "random" => {
                    if let Some(config) = start_config.get("config") {
                        StartConfigBuilder::Random(RandomStartConfigBuilder::from_json(
                            config, seed,
                        )?)
                    } else {
                        let mut default_builder = RandomStartConfigBuilder::new();
                        default_builder.seed = Some(seed);
                        StartConfigBuilder::Random(default_builder)
                    }
                }
                _ => return Err(ConfigError::JsonError(config_type.to_string())),
            });
        }

        if let Some(task_config) = value.get("task_config") {
            builder.task_config = Some(TaskConfigBuilder::from_json(task_config)?);
        }

        Ok(builder)
    }
}

impl AircraftBuilder for DubinsAircraftConfigBuilder {
    fn build(&self) -> Result<AircraftConfig, ConfigError> {
        info!("Building DubinsAircraftConfig");
        let default_config = DubinsAircraftConfig::default();

        let start_config = match &self.start_config {
            Some(StartConfigBuilder::Fixed(fixed_config)) => {
                Some(StartConfig::Fixed(fixed_config.build()))
            }
            Some(StartConfigBuilder::Random(random_config)) => {
                let config = if let Some(seed) = self.seed {
                    info!("Using master seed {} for random_start_config", seed);
                    random_config.clone().build_with_seed(seed)
                } else {
                    info!("No seed provided, using default");
                    random_config.clone().build()
                };
                Some(StartConfig::Random(config))
            }
            None => None,
        };

        let task_config = match &self.task_config {
            Some(task_config) => task_config.build()?,
            None => TaskType::default(),
        };

        Ok(AircraftConfig::Dubins(DubinsAircraftConfig {
            name: self
                .name
                .clone()
                .unwrap_or_else(|| "unnamed_dubins".to_string()),
            max_speed: self.max_speed.unwrap_or(default_config.max_speed),
            min_speed: self.min_speed.unwrap_or(default_config.min_speed),
            acceleration: self.acceleration.unwrap_or(default_config.acceleration),
            max_bank_angle: self.max_bank_angle.unwrap_or(default_config.max_bank_angle),
            max_turn_rate: self.max_turn_rate.unwrap_or(default_config.max_turn_rate),
            max_climb_rate: self.max_climb_rate.unwrap_or(default_config.max_climb_rate),
            max_descent_rate: self
                .max_descent_rate
                .unwrap_or(default_config.max_descent_rate),
            start_config: start_config.unwrap_or(default_config.start_config),
            task_config,
        }))
    }
}

impl FullAircraftConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(value: &Value, seed: u64) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        builder.name = value.get("name").and_then(|v| v.as_str()).map(String::from);
        
        // Parse trim configuration if present
        if let Some(trim_config) = value.get("trim_condition") {
            let mut trim_condition = TrimConditionConfig::default();
            
            // Parse airspeed
            if let Some(airspeed) = trim_config.get("airspeed").and_then(|v| v.as_f64()) {
                trim_condition.airspeed = airspeed;
            }
            
            // Parse condition type
            if let Some(condition_type) = trim_config.get("condition_type").and_then(|v| v.as_str()) {
                trim_condition.condition_type = condition_type.to_string();
            }
            
            // Parse gamma for climb
            if let Some(gamma) = trim_config.get("gamma").and_then(|v| v.as_f64()) {
                trim_condition.gamma = Some(gamma);
            }
            
            // Parse bank angle for turn
            if let Some(bank_angle) = trim_config.get("bank_angle").and_then(|v| v.as_f64()) {
                trim_condition.bank_angle = Some(bank_angle);
            }
            
            builder.trim_condition = Some(trim_condition);
        }

        if let Some(config) = value.get("config") {
            // Aircraft type
            if let Some(type_str) = config.get("ac_type").and_then(|t| t.as_str()) {
                builder.ac_type = Some(match type_str {
                    "twin_otter" => AircraftType::TwinOtter,
                    "f4_phantom" => AircraftType::F4Phantom,
                    _ => AircraftType::GenericTransport,
                });
            }

            // Parse mass configuration
            if let Some(mass_config) = config.get("mass") {
                builder.mass = parse_mass_json(mass_config)?;
            }

            // Parse geometry configuration
            if let Some(geom_config) = config.get("geometry") {
                builder.geometry = parse_geometry_json(geom_config)?;
            }

            // Handle aero coefficients based on aircraft type
            builder.aero_coef = Some(
                match builder
                    .ac_type
                    .as_ref()
                    .unwrap_or(&AircraftType::GenericTransport)
                {
                    AircraftType::TwinOtter => AircraftAeroCoefficients::twin_otter(),
                    AircraftType::F4Phantom => AircraftAeroCoefficients::f4_phantom(),
                    _ => AircraftAeroCoefficients::generic_transport(),
                },
            );

            // Handle propulsion config based on aircraft_type
            builder.propulsion_config = Some(
                match builder
                    .ac_type
                    .as_ref()
                    .unwrap_or(&AircraftType::GenericTransport)
                {
                    AircraftType::TwinOtter => PropulsionConfig::twin_otter(),
                    AircraftType::F4Phantom => PropulsionConfig::f4_phantom(),
                    _ => PropulsionConfig::single_engine(PowerplantConfig::default()),
                },
            );

            // Create RandomStartConfigBuilder with either provided config or defaults
            if let Some(start_config) = value.get("start_config") {
                let config_type = start_config
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConfigError::MissingRequired("start_config.type".into()))?;

                builder.start_config = Some(match config_type {
                    "fixed" => {
                        if let Some(config) = start_config.get("config") {
                            StartConfigBuilder::Fixed(FixedStartConfigBuilder::from_json(config)?)
                        } else {
                            return Err(ConfigError::MissingRequired("start_config.config".into()));
                        }
                    }
                    "random" => {
                        if let Some(config) = start_config.get("config") {
                            StartConfigBuilder::Random(RandomStartConfigBuilder::from_json(
                                config, seed,
                            )?)
                        } else {
                            let mut default_builder = RandomStartConfigBuilder::new();
                            default_builder.seed = Some(seed);
                            StartConfigBuilder::Random(default_builder)
                        }
                    }
                    _ => return Err(ConfigError::JsonError(config_type.to_string())),
                });
            }

            if let Some(task_config) = value.get("task_config") {
                builder.task_config = Some(TaskConfigBuilder::from_json(task_config)?);
            }
        }

        Ok(builder)
    }
}

impl AircraftBuilder for FullAircraftConfigBuilder {
    fn build(&self) -> Result<AircraftConfig, ConfigError> {
        let ac_type = self
            .ac_type
            .clone()
            .unwrap_or(AircraftType::GenericTransport);
        let name = self.name.clone().unwrap_or_else(|| {
            format!(
                "unnamed_{}",
                match ac_type {
                    AircraftType::TwinOtter => "twin_otter",
                    AircraftType::F4Phantom => "f4",
                    _ => "generic",
                }
            )
        });

        let start_config = match &self.start_config {
            Some(StartConfigBuilder::Fixed(fixed_config)) => {
                Some(StartConfig::Fixed(fixed_config.build()))
            }
            Some(StartConfigBuilder::Random(random_config)) => {
                let config = if let Some(seed) = self.seed {
                    info!("Using master seed {} for random_start_config", seed);
                    random_config.clone().build_with_seed(seed)
                } else {
                    info!("No seed provided, using default");
                    random_config.clone().build()
                };
                Some(StartConfig::Random(config))
            }
            None => None,
        };

        let task_config = match &self.task_config {
            Some(task_config) => task_config.build()?,
            None => TaskType::default(),
        };

        Ok(AircraftConfig::Full(FullAircraftConfig {
            name,
            ac_type: ac_type.clone(),
            mass: self.mass.clone().unwrap_or_else(|| match ac_type {
                AircraftType::TwinOtter => MassModel::twin_otter(),
                AircraftType::F4Phantom => MassModel::f4_phantom(),
                _ => MassModel::generic_transport(),
            }),
            geometry: self.geometry.clone().unwrap_or_else(|| match ac_type {
                AircraftType::TwinOtter => AircraftGeometry::twin_otter(),
                AircraftType::F4Phantom => AircraftGeometry::f4_phantom(),
                _ => AircraftGeometry::generic_transport(),
            }),
            aero_coef: self.aero_coef.clone().unwrap_or_else(|| match ac_type {
                AircraftType::TwinOtter => AircraftAeroCoefficients::twin_otter(),
                AircraftType::F4Phantom => AircraftAeroCoefficients::f4_phantom(),
                _ => AircraftAeroCoefficients::generic_transport(),
            }),
            propulsion: self
                .propulsion_config
                .clone()
                .unwrap_or_else(|| match ac_type {
                    AircraftType::TwinOtter => PropulsionConfig::twin_otter(),
                    AircraftType::F4Phantom => PropulsionConfig::f4_phantom(),
                    _ => PropulsionConfig::single_engine(PowerplantConfig::default()),
                }),
            start_config: start_config.unwrap_or_default(),
            task_config,
            trim_condition: self.trim_condition.as_ref().map(|tc| tc.to_trim_condition()),
        }))
    }
}

// Helper functions to parse configuration
fn parse_mass_json(value: &Value) -> Result<Option<MassModel>, ConfigError> {
    let mass = value.get("mass").and_then(|v| v.as_f64());
    let ixx = value.get("ixx").and_then(|v| v.as_f64());
    let iyy = value.get("iyy").and_then(|v| v.as_f64());
    let izz = value.get("izz").and_then(|v| v.as_f64());
    let ixz = value.get("ixz").and_then(|v| v.as_f64());

    match (mass, ixx, iyy, izz, ixz) {
        (Some(mass), Some(ixx), Some(iyy), Some(izz), Some(ixz)) => {
            Ok(Some(MassModel::new(mass, ixx, iyy, izz, ixz)))
        }
        _ => Ok(None),
    }
}

fn parse_geometry_json(value: &Value) -> Result<Option<AircraftGeometry>, ConfigError> {
    let wing_area = value.get("wing_area").and_then(|v| v.as_f64());
    let wing_span = value.get("wing_span").and_then(|v| v.as_f64());
    let mac = value.get("mac").and_then(|v| v.as_f64());

    match (wing_area, wing_span, mac) {
        (Some(wing_area), Some(wing_span), Some(mac)) => {
            Ok(Some(AircraftGeometry::new(wing_area, wing_span, mac)))
        }
        _ => Ok(None),
    }
}

pub fn create_aircraft_builder(
    value: &Value,
    seed: u64,
) -> Result<AircraftAgentBuilder, ConfigError> {
    let aircraft_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::MissingRequired("aircraft type".into()))?;

    let action_type = value
        .get("action_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::MissingRequired("action type".into()))?;

    let observation_type = value
        .get("observation_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::MissingRequired("observation type".into()))?;

    let task_config = if let Some(task_config) = value.get("task_config") {
        Some(TaskConfigBuilder::from_json(task_config)?)
    } else {
        None
    };

    let aircraft_builder = match aircraft_type {
        "dubins" => {
            let mut builder = DubinsAircraftConfigBuilder::from_json(value, seed)?;
            if let Some(task_config) = task_config {
                builder.task_config = Some(task_config);
            }
            AircraftBuilderEnum::Dubins(builder)
        }
        "full" => {
            let mut builder = FullAircraftConfigBuilder::from_json(value, seed)?;
            if let Some(task_config) = task_config {
                builder.task_config = Some(task_config);
            }
            AircraftBuilderEnum::Full(builder)
        }
        _ => return Err(ConfigError::InvalidAircraftType(aircraft_type.to_string())),
    };

    let action_builder = match action_type {
        // No differnce between Continuous and Discrete anymore, could simplify
        "Continuous" => match aircraft_type {
            "dubins" => ActionSpaceBuilder::new().act_space(ActionSpace::new_continuous_dubins()),
            "full" => ActionSpaceBuilder::new().act_space(ActionSpace::new_continuous_full()),
            _ => return Err(ConfigError::InvalidActionType(action_type.to_string())),
        },
        "Discrete" => match aircraft_type {
            "dubins" => ActionSpaceBuilder::new().act_space(ActionSpace::new_discrete_dubins()),
            "full" => ActionSpaceBuilder::new().act_space(ActionSpace::new_discrete_full()),
            _ => return Err(ConfigError::InvalidActionType(action_type.to_string())),
        },
        _ => return Err(ConfigError::InvalidActionType(action_type.to_string())),
    };

    let observation_builder = match observation_type {
        "Continuous" => match aircraft_type {
            "dubins" => ObservationSpaceBuilder::new().obs_space(ObservationSpace::Continuous(
                ContinuousObservationSpace::DubinsAircraft,
            )),
            "full" => ObservationSpaceBuilder::new().obs_space(ObservationSpace::Continuous(
                ContinuousObservationSpace::FullAircraft,
            )),
            _ => return Err(ConfigError::InvalidAircraftType(aircraft_type.to_string())),
        },
        _ => {
            return Err(ConfigError::InvalidObservationType(
                observation_type.to_string(),
            ))
        }
    };

    Ok(AircraftAgentBuilder {
        aircraft_builder,
        observation_builder,
        action_builder,
    })
}
