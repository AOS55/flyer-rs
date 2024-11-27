use crate::utils::errors::SimError;
use nalgebra::Matrix3;
use serde::{Deserialize, Serialize};

/// Complete aircraft configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftConfig {
    /// Aircraft name
    pub name: String,
    /// Aircraft mass [kg]
    pub mass: f64,
    /// Aircraft inertia array [kg⋅m²]
    pub inertia: Matrix3<f64>,
    /// Aircraft wing area [m²]
    pub wing_area: f64,
    /// Aircraft wing span [m]
    pub wing_span: f64,
    /// Aircraft mean aerodynamic chord [m]
    pub mac: f64,
    /// Aircraft aerodynamic parameters
    pub aero: AerodynamicConfig,
    /// Aircraft propulsion parameters
    pub propulsion: PropulsionConfig,
}

/// Aerodynamic configuration including all coefficients
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AerodynamicConfig {
    /// Drag coefficients
    pub drag: DragConfig,
    /// Side force coefficients
    pub side_force: SideForceConfig,
    /// Lift coefficients
    pub lift: LiftConfig,
    /// Roll moment coefficients
    pub roll: RollConfig,
    /// Pitch moment coefficients
    pub pitch: PitchConfig,
    /// Yaw moment coefficients
    pub yaw: YawConfig,
}

/// Aerodynamic drag (D) parameters
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct DragConfig {
    /// 0 alpha drag
    pub c_D_0: f64,
    /// drag due to alpha
    pub c_D_alpha: f64,
    /// drag due to alpha.q
    pub c_D_alpha_q: f64,
    /// drag due to alpha.delta_elevator
    pub c_D_alpha_deltae: f64,
    /// drag due to alpha^2
    pub c_D_alpha2: f64,
    /// drag due to alpha^2.q
    pub c_D_alpha2_q: f64,
    /// drag due to alpha^2.delta_elevator
    pub c_D_alpha2_deltae: f64,
    /// drag due to alpha^3
    pub c_D_alpha3: f64,
    /// drag due to alpha^3.q
    pub c_D_alpha3_q: f64,
    /// drag due to alpha^4
    pub c_D_alpha4: f64,
}

/// Aerodynamic sideforce (Y) parameters
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct SideForceConfig {
    /// sideforce due to beta
    pub c_Y_beta: f64,
    /// sideforce due to p
    pub c_Y_p: f64,
    /// sideforce due to r
    pub c_Y_r: f64,
    /// sideforce due to delta_aileron
    pub c_Y_deltaa: f64,
    /// sideforce due to delta_rudder
    pub c_Y_deltar: f64,
}

/// Aerodynamic lift (L) parameters
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct LiftConfig {
    /// 0 alpha lift
    pub c_L_0: f64,
    /// lift due to angle of attack
    pub c_L_alpha: f64,
    /// lift due to pitch rate
    pub c_L_q: f64,
    /// lift due to delta_e
    pub c_L_deltae: f64,
    /// lift due to alpha.q
    pub c_L_alpha_q: f64,
    /// lift due to alpha^2
    pub c_L_alpha2: f64,
    /// lift due to alpha^3
    pub c_L_alpha3: f64,
    /// lift due to alpha^4
    pub c_L_alpha4: f64,
}

/// Aerodynamic roll moment (l)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RollConfig {
    /// roll moment due to beta
    pub c_l_beta: f64,
    /// roll moment due to p
    pub c_l_p: f64,
    /// roll moment due to r
    pub c_l_r: f64,
    /// roll moment due to delta_aileron
    pub c_l_deltaa: f64,
    /// roll moment due to delta_rudder
    pub c_l_deltar: f64,
}

/// Aerodynamic pitch moment (m)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PitchConfig {
    /// 0 alpha pitch moment
    pub c_m_0: f64,
    /// pitch moment due to alpha
    pub c_m_alpha: f64,
    /// pitch moment due to q
    pub c_m_q: f64,
    /// pitch moment due to delta_e
    pub c_m_deltae: f64,
    /// pitch moment due to alpha.q
    pub c_m_alpha_q: f64,
    /// pitch moment due to alpha^2.q
    pub c_m_alpha2_q: f64,
    /// pitch moment due to alpha^2.delta_e
    pub c_m_alpha2_deltae: f64,
    /// pitch moment due to alpha^3.q
    pub c_m_alpha3_q: f64,
    /// pitch moment due to alpha^3.delta_e
    pub c_m_alpha3_deltae: f64,
    /// pitch moment due to alpha^4
    pub c_m_alpha4: f64,
}

/// Aerodynamic yaw moment
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct YawConfig {
    /// yaw moment due to beta
    pub c_n_beta: f64,
    /// yaw moment due to p
    pub c_n_p: f64,
    /// yaw moment due to r
    pub c_n_r: f64,
    /// yaw moment due to delta_a
    pub c_n_deltaa: f64,
    /// yaw moment due to delta_r
    pub c_n_deltar: f64,
    /// yaw moment due to beta^2
    pub c_n_beta2: f64,
    /// yaw moment due to beta^3
    pub c_n_beta3: f64,
}

/// Engine/propulsion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropulsionConfig {
    /// Name of the power-plant/engine
    pub name: String,
    /// Maximum shaft-power [W]
    pub shaft_power: f64,
    /// Maximum velocity [m/s]
    pub max_velocity: f64,
    /// Maximum efficiency
    pub efficiency: f64,
}

impl Default for PropulsionConfig {
    fn default() -> Self {
        Self {
            name: "PT6".to_string(),
            shaft_power: 2.0 * 1.12e6,
            max_velocity: 40.0,
            efficiency: 0.6,
        }
    }
}

impl AircraftConfig {
    /// Load an aircraft configuration from a YAML file
    pub fn from_yaml(path: &str) -> Result<Self, SimError> {
        let file = std::fs::File::open(path)?;
        let config: AircraftConfig = serde_yaml::from_reader(file)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the aircraft configuration
    pub fn validate(&self) -> Result<(), SimError> {
        // Basic parameter validation
        if self.mass <= 0.0 {
            return Err(SimError::InvalidConfig("Mass must be positive".into()));
        }
        if self.wing_area <= 0.0 {
            return Err(SimError::InvalidConfig("Wing area must be positive".into()));
        }
        if self.wing_span <= 0.0 {
            return Err(SimError::InvalidConfig("Wing span must be positive".into()));
        }
        if self.mac <= 0.0 {
            return Err(SimError::InvalidConfig(
                "Mean aerodynamic chord must be positive".into(),
            ));
        }

        // Validate inertia matrix is positive definite
        if self.inertia.determinant() <= 0.0 {
            return Err(SimError::InvalidConfig(
                "Inertia matrix must be positive definite".into(),
            ));
        }

        // Propulsion validation
        if self.propulsion.shaft_power <= 0.0 {
            return Err(SimError::InvalidConfig(
                "Shaft power must be positive".into(),
            ));
        }
        if self.propulsion.max_velocity <= 0.0 {
            return Err(SimError::InvalidConfig(
                "Maximum velocity must be positive".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.propulsion.efficiency) {
            return Err(SimError::InvalidConfig(
                "Efficiency must be between 0 and 1".into(),
            ));
        }

        Ok(())
    }
}
