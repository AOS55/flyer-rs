use crate::ecs::component::Component;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AerodynamicsComponent {
    pub geometry: AircraftGeometry,
    pub air_data: AirData,
    pub coefficients: AeroCoefficients,
    pub control_surfaces: ControlSurfaces,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftGeometry {
    pub wing_area: f64,
    pub wing_span: f64,
    pub mean_aerodynamic_chord: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirData {
    pub true_airspeed: f64,
    pub alpha: f64,
    pub beta: f64,
    pub dynamic_pressure: f64,
    pub density: f64,
    pub relative_velocity: Vector3<f64>,
    pub wind_velocity: Vector3<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AeroCoefficients {
    pub drag: DragCoefficients,
    pub lift: LiftCoefficients,
    pub side_force: SideForceCoefficients,
    pub roll: RollCoefficients,
    pub pitch: PitchCoefficients,
    pub yaw: YawCoefficients,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlSurfaces {
    pub elevator: f64,
    pub aileron: f64,
    pub rudder: f64,
    pub flaps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragCoefficients {
    pub c_d_0: f64,
    pub c_d_alpha: f64,
    pub c_d_alpha_q: f64,
    pub c_d_alpha_deltae: f64,
    pub c_d_alpha2: f64,
    pub c_d_alpha2_q: f64,
    pub c_d_alpha2_deltae: f64,
    pub c_d_alpha3: f64,
    pub c_d_alpha3_q: f64,
    pub c_d_alpha4: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiftCoefficients {
    pub c_l_0: f64,
    pub c_l_alpha: f64,
    pub c_l_q: f64,
    pub c_l_deltae: f64,
    pub c_l_alpha_q: f64,
    pub c_l_alpha2: f64,
    pub c_l_alpha3: f64,
    pub c_l_alpha4: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideForceCoefficients {
    pub c_y_beta: f64,
    pub c_y_p: f64,
    pub c_y_r: f64,
    pub c_y_deltaa: f64,
    pub c_y_deltar: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollCoefficients {
    pub c_l_beta: f64,
    pub c_l_p: f64,
    pub c_l_r: f64,
    pub c_l_deltaa: f64,
    pub c_l_deltar: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchCoefficients {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YawCoefficients {
    pub c_n_beta: f64,
    pub c_n_p: f64,
    pub c_n_r: f64,
    pub c_n_deltaa: f64,
    pub c_n_deltar: f64,
    pub c_n_beta2: f64,
    pub c_n_beta3: f64,
}

impl Component for AerodynamicsComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for AerodynamicsComponent {
    fn default() -> Self {
        Self {
            geometry: AircraftGeometry::default(),
            air_data: AirData::default(),
            coefficients: AeroCoefficients::default(),
            control_surfaces: ControlSurfaces::default(),
        }
    }
}

impl Default for AircraftGeometry {
    fn default() -> Self {
        Self {
            wing_area: 16.0,
            wing_span: 10.0,
            mean_aerodynamic_chord: 1.6,
        }
    }
}

impl Default for AirData {
    fn default() -> Self {
        Self {
            true_airspeed: 0.0,
            alpha: 0.0,
            beta: 0.0,
            dynamic_pressure: 0.0,
            density: 1.225,
            relative_velocity: Vector3::zeros(),
            wind_velocity: Vector3::zeros(),
        }
    }
}

impl Default for AeroCoefficients {
    fn default() -> Self {
        Self {
            drag: DragCoefficients::default(),
            lift: LiftCoefficients::default(),
            side_force: SideForceCoefficients::default(),
            roll: RollCoefficients::default(),
            pitch: PitchCoefficients::default(),
            yaw: YawCoefficients::default(),
        }
    }
}

impl Default for ControlSurfaces {
    fn default() -> Self {
        Self {
            elevator: 0.0,
            aileron: 0.0,
            rudder: 0.0,
            flaps: 0.0,
        }
    }
}

impl Default for DragCoefficients {
    fn default() -> Self {
        Self {
            c_d_0: 0.108,
            c_d_alpha: 0.138,
            c_d_alpha_q: -54.05,
            c_d_alpha_deltae: 0.111,
            c_d_alpha2: 2.988,
            c_d_alpha2_q: 302.1,
            c_d_alpha2_deltae: 0.156,
            c_d_alpha3: -7.743,
            c_d_alpha3_q: -218.8,
            c_d_alpha4: 11.77,
        }
    }
}

impl Default for LiftCoefficients {
    fn default() -> Self {
        Self {
            c_l_0: 0.215,
            c_l_alpha: 4.370,
            c_l_q: 25.05,
            c_l_deltae: 0.291,
            c_l_alpha_q: 52.78,
            c_l_alpha2: 16.62,
            c_l_alpha3: -87.67,
            c_l_alpha4: 90.41,
        }
    }
}

impl Default for SideForceCoefficients {
    fn default() -> Self {
        Self {
            c_y_beta: -0.885,
            c_y_p: -0.090,
            c_y_r: 1.697,
            c_y_deltaa: -0.051,
            c_y_deltar: -0.193,
        }
    }
}

impl Default for RollCoefficients {
    fn default() -> Self {
        Self {
            c_l_beta: -0.112,
            c_l_p: -0.413,
            c_l_r: 0.191,
            c_l_deltaa: -0.206,
            c_l_deltar: 0.116,
        }
    }
}

impl Default for PitchCoefficients {
    fn default() -> Self {
        Self {
            c_m_0: 0.057,
            c_m_alpha: -1.419,
            c_m_q: -27.95,
            c_m_deltae: -1.626,
            c_m_alpha_q: 100.7,
            c_m_alpha2_q: -759.2,
            c_m_alpha2_deltae: 7.664,
            c_m_alpha3_q: 1103.0,
            c_m_alpha3_deltae: -8.121,
            c_m_alpha4: 2.468,
        }
    }
}

impl Default for YawCoefficients {
    fn default() -> Self {
        Self {
            c_n_beta: 0.088,
            c_n_p: -0.043,
            c_n_r: -0.426,
            c_n_deltaa: 0.023,
            c_n_deltar: -0.087,
            c_n_beta2: 0.337,
            c_n_beta3: -0.766,
        }
    }
}
