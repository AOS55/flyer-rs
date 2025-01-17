use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::f64::consts::PI;

use crate::components::tasks::{
    ControlParams, ControlType, GoalParams, GoalRewardType, LandingParams, RunwayParams, TaskType,
    TrajectoryMotionPrimitive, TrajectoryParams, TurnDirection,
};
use crate::server::config::errors::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum TaskConfigBuilder {
    Control(ControlTaskConfigBuilder),
    Goal(GoalTaskConfigBuilder),
    Trajectory(TrajectoryTaskConfigBuilder),
    Runway(RunwayTaskConfigBuilder),
    Landing(LandingTaskConfigBuilder),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlTaskConfigBuilder {
    pub target: Option<f64>,
    pub tolerance: Option<f64>,
    pub control_type: Option<ControlType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTaskConfigBuilder {
    pub position: Option<Vector3<f64>>,
    pub reward_type: Option<GoalRewardType>,
    pub tolerance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryTaskConfigBuilder {
    pub motion_primitive: Option<TrajectoryMotionPrimitive>,
    pub target_velocity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunwayTaskConfigBuilder {
    pub position: Option<Vector3<f64>>,
    pub heading: Option<f64>,
    pub width: Option<f64>,
    pub length: Option<f64>,
    pub glideslope: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandingTaskConfigBuilder {
    target_position: Option<Vector3<f64>>,
    max_landing_speed: Option<f64>,
    max_descent_rate: Option<f64>,
    max_bank_angle: Option<f64>,
    max_landing_distance: Option<f64>,
    landing_complete_height: Option<f64>,
}

impl Default for ControlTaskConfigBuilder {
    fn default() -> Self {
        Self {
            target: Some(0.0),
            tolerance: Some(0.1),
            control_type: Some(ControlType::default()),
        }
    }
}

impl Default for GoalTaskConfigBuilder {
    fn default() -> Self {
        Self {
            position: Some(Vector3::new(0.0, 0.0, 0.0)),
            reward_type: Some(GoalRewardType::Dense),
            tolerance: Some(1.0),
        }
    }
}

impl Default for TrajectoryTaskConfigBuilder {
    fn default() -> Self {
        Self {
            motion_primitive: Some(TrajectoryMotionPrimitive::StraightAndLevel {
                target_distance: 100.0,
            }),
            target_velocity: Some(20.0),
        }
    }
}

impl Default for RunwayTaskConfigBuilder {
    fn default() -> Self {
        Self {
            position: Some(Vector3::new(0.0, 0.0, 0.0)),
            heading: Some(0.0),
            width: Some(30.0),
            length: Some(1000.0),
            glideslope: Some(3.0),
        }
    }
}

impl Default for LandingTaskConfigBuilder {
    fn default() -> Self {
        Self {
            target_position: None,
            max_landing_speed: Some(25.0),      // m/s
            max_descent_rate: Some(3.0),        // m/s
            max_bank_angle: Some(PI / 6.0),     // 30 degrees
            max_landing_distance: Some(200.0),  // meters
            landing_complete_height: Some(0.5), // meters
        }
    }
}

impl TaskConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let reward_type = value
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::MissingRequired("reward type".into()))?;

        let config = value
            .get("config")
            .ok_or_else(|| ConfigError::MissingRequired("reward config".into()))?;

        match reward_type {
            "Control" => Ok(TaskConfigBuilder::Control(
                ControlTaskConfigBuilder::from_json(config)?,
            )),
            "Goal" => Ok(TaskConfigBuilder::Goal(GoalTaskConfigBuilder::from_json(
                config,
            )?)),
            "Trajectory" => Ok(TaskConfigBuilder::Trajectory(
                TrajectoryTaskConfigBuilder::from_json(config)?,
            )),
            "Runway" => Ok(TaskConfigBuilder::Runway(
                RunwayTaskConfigBuilder::from_json(config)?,
            )),
            "Landing" => Ok(TaskConfigBuilder::Landing(
                LandingTaskConfigBuilder::from_json(config)?,
            )),
            _ => Err(ConfigError::InvalidParameter {
                name: "reward type".into(),
                value: reward_type.into(),
            }),
        }
    }

    pub fn build(&self) -> Result<TaskType, ConfigError> {
        match self {
            TaskConfigBuilder::Control(builder) => builder.build(),
            TaskConfigBuilder::Goal(builder) => builder.build(),
            TaskConfigBuilder::Trajectory(builder) => builder.build(),
            TaskConfigBuilder::Runway(builder) => builder.build(),
            TaskConfigBuilder::Landing(builder) => builder.build(),
        }
    }
}

impl ControlTaskConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::default();

        if let Some(target) = value.get("target").and_then(|v| v.as_f64()) {
            builder.target = Some(target);
        }

        if let Some(tolerance) = value.get("tolerance").and_then(|v| v.as_f64()) {
            builder.tolerance = Some(tolerance);
        }

        if let Some(control_type) = value.get("control_type").and_then(|v| v.as_str()) {
            builder.control_type = Some(match control_type {
                "Altitude" => ControlType::Altitude,
                "Heading" => ControlType::Heading,
                "Speed" => ControlType::Speed,
                "Pitch" => ControlType::Pitch,
                "Roll" => ControlType::Roll,
                _ => {
                    return Err(ConfigError::InvalidParameter {
                        name: "control_type".into(),
                        value: control_type.into(),
                    })
                }
            });
        }

        Ok(builder)
    }

    pub fn build(&self) -> Result<TaskType, ConfigError> {
        let params = ControlParams {
            target: self.target.unwrap(),
            tolerance: self.tolerance.unwrap(),
            control_type: self.control_type.unwrap(),
        };

        Ok(TaskType::Control(params))
    }
}

impl GoalTaskConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::default();

        if let Some(pos) = value.get("position") {
            builder.position = Some(Vector3::new(
                pos.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                pos.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                pos.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
            ));
        }

        if let Some(tolerance) = value.get("tolerance").and_then(|v| v.as_f64()) {
            builder.tolerance = Some(tolerance);
        }

        if let Some(reward_type) = value.get("reward_type").and_then(|v| v.as_str()) {
            builder.reward_type = Some(match reward_type {
                "Sparse" => GoalRewardType::Sparse,
                "Dense" => GoalRewardType::Dense,
                _ => {
                    return Err(ConfigError::InvalidParameter {
                        name: "reward_type".into(),
                        value: reward_type.into(),
                    })
                }
            });
        }

        Ok(builder)
    }

    pub fn build(&self) -> Result<TaskType, ConfigError> {
        let params = GoalParams {
            position: self.position.unwrap(),
            reward_type: self.reward_type.unwrap(),
            tolerance: self.tolerance.unwrap(),
        };

        Ok(TaskType::Goal(params))
    }
}

impl TrajectoryTaskConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::default();

        if let Some(target_velocity) = value.get("target_velocity").and_then(|v| v.as_f64()) {
            builder.target_velocity = Some(target_velocity);
        }

        if let Some(motion) = value.get("motion_type") {
            let motion_type = motion.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
                ConfigError::MissingRequired("trajectory motion primitive type".into())
            })?;

            let motion_primitive = match motion_type {
                "StraightAndLevel" => {
                    let target_distance = motion
                        .get("target_distance")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(100.0);
                    TrajectoryMotionPrimitive::StraightAndLevel { target_distance }
                }
                "CoordinatedTurn" => {
                    let turn_radius = motion
                        .get("turn_radius")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(100.0);
                    let turn_angle = motion
                        .get("turn_angle")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(90.0);
                    let direction =
                        if motion.get("direction").and_then(|v| v.as_str()) == Some("Left") {
                            TurnDirection::Left
                        } else {
                            TurnDirection::Right
                        };
                    TrajectoryMotionPrimitive::CoordinatedTurn {
                        turn_radius,
                        turn_angle,
                        direction,
                    }
                }
                "Climb" => {
                    let target_climb_rate = motion
                        .get("target_climb_rate")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(5.0);
                    TrajectoryMotionPrimitive::Climb { target_climb_rate }
                }
                "Descend" => {
                    let target_descent_rate = motion
                        .get("target_descent_rate")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(5.0);
                    TrajectoryMotionPrimitive::Descend {
                        target_descent_rate,
                    }
                }
                "StraightAndTurn" => {
                    let straight_distance = motion
                        .get("straight_distance")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(100.0);
                    let turn_radius = motion
                        .get("turn_radius")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(100.0);
                    let turn_angle = motion
                        .get("turn_angle")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(90.0);
                    let direction =
                        if motion.get("direction").and_then(|v| v.as_str()) == Some("Left") {
                            TurnDirection::Left
                        } else {
                            TurnDirection::Right
                        };
                    TrajectoryMotionPrimitive::StraightAndTurn {
                        straight_distance,
                        turn_radius,
                        turn_angle,
                        direction,
                    }
                }
                _ => {
                    return Err(ConfigError::InvalidParameter {
                        name: "motion_type".into(),
                        value: motion_type.into(),
                    })
                }
            };

            builder.motion_primitive = Some(motion_primitive);
        }

        Ok(builder)
    }

    pub fn build(&self) -> Result<TaskType, ConfigError> {
        let params = TrajectoryParams {
            motion_type: self.motion_primitive.clone().unwrap(),
            target_velocity: self.target_velocity.unwrap(),
        };

        Ok(TaskType::Trajectory(params))
    }
}

impl RunwayTaskConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::default();

        if let Some(pos) = value.get("position") {
            builder.position = Some(Vector3::new(
                pos.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                pos.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                pos.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
            ));
        }

        if let Some(heading) = value.get("heading").and_then(|v| v.as_f64()) {
            builder.heading = Some(heading);
        }
        if let Some(width) = value.get("width").and_then(|v| v.as_f64()) {
            builder.width = Some(width);
        }
        if let Some(length) = value.get("length").and_then(|v| v.as_f64()) {
            builder.length = Some(length);
        }
        if let Some(glideslope) = value.get("glideslope").and_then(|v| v.as_f64()) {
            builder.glideslope = Some(glideslope);
        }

        Ok(builder)
    }

    pub fn build(&self) -> Result<TaskType, ConfigError> {
        let params = RunwayParams {
            position: self.position.unwrap(),
            heading: self.heading.unwrap(),
            width: self.width.unwrap(),
            length: self.length.unwrap(),
            glideslope: self.glideslope.unwrap(),
        };

        Ok(TaskType::Runway(params))
    }
}

impl LandingTaskConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::default();

        if let Some(pos) = value.get("target_position") {
            builder.target_position = Some(Vector3::new(
                pos.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                pos.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                pos.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
            ));
        }

        if let Some(speed) = value.get("max_landing_speed").and_then(|v| v.as_f64()) {
            builder.max_landing_speed = Some(speed);
        }

        if let Some(rate) = value.get("max_descent_rate").and_then(|v| v.as_f64()) {
            builder.max_descent_rate = Some(rate);
        }

        if let Some(angle) = value.get("max_bank_angle").and_then(|v| v.as_f64()) {
            builder.max_bank_angle = Some(angle);
        }

        if let Some(distance) = value.get("max_landing_distance").and_then(|v| v.as_f64()) {
            builder.max_landing_distance = Some(distance);
        }

        if let Some(height) = value
            .get("landing_complete_height")
            .and_then(|v| v.as_f64())
        {
            builder.landing_complete_height = Some(height);
        }

        Ok(builder)
    }

    pub fn build(&self) -> Result<TaskType, ConfigError> {
        let params = LandingParams {
            target_position: self.target_position,
            max_landing_speed: self.max_landing_speed.unwrap_or_default(),
            max_descent_rate: self.max_descent_rate.unwrap_or_default(),
            max_bank_angle: self.max_bank_angle.unwrap_or_default(),
            max_landing_distance: self.max_landing_distance.unwrap_or_default(),
            landing_complete_height: self.landing_complete_height.unwrap_or_default(),
        };

        Ok(TaskType::ForcedLanding(params))
    }
}
