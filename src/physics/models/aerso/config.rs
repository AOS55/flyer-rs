use crate::vehicles::aircraft::systems::{
    DragData, LiftData, PitchData, RollData, SideForceData, YawData,
};

use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AersoConfig {
    // Basic parameters
    pub mass: f64,
    pub inertia: Matrix3<f64>,

    // Aerodynamics configuration
    pub aero_coefficients: AeroCoefficients,

    // Propulsion configuration
    pub engine_params: EngineParameters,

    // Environmental settings
    pub atmosphere_model: AtmosphereModel,
    pub wind_model: WindModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AeroCoefficients {
    pub drag_data: DragData,
    pub side_force_data: SideForceData,
    pub lift_data: LiftData,
    pub roll_data: RollData,
    pub pitch_data: PitchData,
    pub yaw_data: YawData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineParameters {
    pub max_power: f64,
    pub efficiency: f64,
    pub max_velocity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtmosphereModel {
    Constant,
    Standard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindModelConfig {
    Constant(Vector3<f64>),
    LogWind {
        d: f64,
        z0: f64,
        u_star: f64,
        bearing: f64,
    },
    PowerWind {
        u_r: f64,
        z_r: f64,
        bearing: f64,
        alpha: f64,
    },
}
