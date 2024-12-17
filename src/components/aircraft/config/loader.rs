use serde::Deserialize;
use thiserror::Error;

use crate::components::aircraft::config::aero_coef::{
    AircraftAeroCoefficients, DragCoefficients, LiftCoefficients, PitchCoefficients,
    RollCoefficients, SideForceCoefficients, YawCoefficients,
};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileError(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Invalid aircraft configuration: {0}")]
    ValidationError(String),
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct RawAircraftConfig {
    /// Aircraft identification
    pub name: String,

    /// Mass properties
    pub mass: f64,
    pub ixx: f64,
    pub iyy: f64,
    pub izz: f64,
    pub ixz: f64,

    /// Geometry
    pub wing_area: f64,
    pub wing_span: f64,
    pub mac: f64,

    /// Drag coefficients
    pub c_D_0: f64,
    pub c_D_alpha: f64,
    pub c_D_alpha_q: f64,
    pub c_D_alpha_deltae: f64,
    pub c_D_alpha2: f64,
    pub c_D_alpha2_q: f64,
    pub c_D_alpha2_deltae: f64,
    pub c_D_alpha3: f64,
    pub c_D_alpha3_q: f64,
    pub c_D_alpha4: f64,

    /// Side-force coefficients
    pub c_Y_beta: f64,
    pub c_Y_p: f64,
    pub c_Y_r: f64,
    pub c_Y_deltaa: f64,
    pub c_Y_deltar: f64,

    /// Lift coefficients
    pub c_L_0: f64,
    pub c_L_alpha: f64,
    pub c_L_q: f64,
    pub c_L_deltae: f64,
    pub c_L_alpha_q: f64,
    pub c_L_alpha2: f64,
    pub c_L_alpha3: f64,
    pub c_L_alpha4: f64,

    /// Roll coefficients
    pub c_l_beta: f64,
    pub c_l_p: f64,
    pub c_l_r: f64,
    pub c_l_deltaa: f64,
    pub c_l_deltar: f64,

    /// Pitch coefficients
    pub c_m_0: f64,
    pub c_m_alpha: f64,
    pub c_m_q: f64,
    pub c_m_deltae: f64,
    pub c_m_alpha_q: f64,
    pub c_m_alpha2_q: f64,
    pub c_m_alpha2_deltae: f64,
    pub c_m_alpha3_q: f64,
    pub c_m_alpha3_deltae: f64,
    pub c_m_alpha4: f64,

    /// Yaw coefficients
    pub c_n_beta: f64,
    pub c_n_p: f64,
    pub c_n_r: f64,
    pub c_n_deltaa: f64,
    pub c_n_deltar: f64,
    pub c_n_beta2: f64,
    pub c_n_beta3: f64,
}

impl AircraftAeroCoefficients {
    pub fn from_raw(raw: &RawAircraftConfig) -> Result<Self, ConfigError> {
        Ok(AircraftAeroCoefficients {
            drag: DragCoefficients {
                c_d_0: raw.c_D_0,
                c_d_alpha: raw.c_D_alpha,
                c_d_alpha_q: raw.c_D_alpha_q,
                c_d_alpha_deltae: raw.c_D_alpha_deltae,
                c_d_alpha2: raw.c_D_alpha2,
                c_d_alpha2_q: raw.c_D_alpha2_q,
                c_d_alpha2_deltae: raw.c_D_alpha2_deltae,
                c_d_alpha3: raw.c_D_alpha3,
                c_d_alpha3_q: raw.c_D_alpha3_q,
                c_d_alpha4: raw.c_D_alpha4,
            },
            lift: LiftCoefficients {
                c_l_0: raw.c_L_0,
                c_l_alpha: raw.c_L_alpha,
                c_l_q: raw.c_L_q,
                c_l_deltae: raw.c_L_deltae,
                c_l_alpha_q: raw.c_L_alpha_q,
                c_l_alpha2: raw.c_L_alpha2,
                c_l_alpha3: raw.c_L_alpha3,
                c_l_alpha4: raw.c_L_alpha4,
            },
            side_force: SideForceCoefficients {
                c_y_beta: raw.c_Y_beta,
                c_y_p: raw.c_Y_p,
                c_y_r: raw.c_Y_r,
                c_y_deltaa: raw.c_Y_deltaa,
                c_y_deltar: raw.c_Y_deltar,
            },
            roll: RollCoefficients {
                c_l_beta: raw.c_l_beta,
                c_l_p: raw.c_l_p,
                c_l_r: raw.c_l_r,
                c_l_deltaa: raw.c_l_deltaa,
                c_l_deltar: raw.c_l_deltar,
            },
            pitch: PitchCoefficients {
                c_m_0: raw.c_m_0,
                c_m_alpha: raw.c_m_alpha,
                c_m_q: raw.c_m_q,
                c_m_deltae: raw.c_m_deltae,
                c_m_alpha_q: raw.c_m_alpha_q,
                c_m_alpha2_q: raw.c_m_alpha2_q,
                c_m_alpha2_deltae: raw.c_m_alpha2_deltae,
                c_m_alpha3_q: raw.c_m_alpha3_q,
                c_m_alpha3_deltae: raw.c_m_alpha3_deltae,
                c_m_alpha4: raw.c_m_alpha4,
            },
            yaw: YawCoefficients {
                c_n_beta: raw.c_n_beta,
                c_n_p: raw.c_n_p,
                c_n_r: raw.c_n_r,
                c_n_deltaa: raw.c_n_deltaa,
                c_n_deltar: raw.c_n_deltar,
                c_n_beta2: raw.c_n_beta2,
                c_n_beta3: raw.c_n_beta3,
            },
        })
    }
}
