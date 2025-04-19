use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AircraftAeroCoefficients {
    pub drag: DragCoefficients,
    pub lift: LiftCoefficients,
    pub side_force: SideForceCoefficients,
    pub roll: RollCoefficients,
    pub pitch: PitchCoefficients,
    pub yaw: YawCoefficients,
}

/// Drag coefficients for full aerodynamic model
#[derive(Component, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DragCoefficients {
    // Base drag coefficient (zero angle of attack).
    pub c_d_0: f64,
    /// Linear drag coefficient with respect to angle of attack (α).
    pub c_d_alpha: f64,
    /// Drag coefficient due to angle of attack rate (α̇) and pitch rate (q).
    pub c_d_alpha_q: f64,
    /// Drag coefficient due to angle of attack and elevator deflection.
    pub c_d_alpha_deltae: f64,
    /// Quadratic drag coefficient with respect to angle of attack squared (α²).
    pub c_d_alpha2: f64,
    /// Quadratic drag coefficient with respect to α² and angle of attack rate (α̇).
    pub c_d_alpha2_q: f64,
    /// Quadratic drag coefficient with respect to α² and elevator deflection.
    pub c_d_alpha2_deltae: f64,
    /// Cubic drag coefficient with respect to α³.
    pub c_d_alpha3: f64,
    /// Cubic drag coefficient with respect to α³ and angle of attack rate (α̇).
    pub c_d_alpha3_q: f64,
    /// Quartic drag coefficient with respect to α⁴.
    pub c_d_alpha4: f64,
}

/// Lift coefficients for full aerodynamic model
#[derive(Component, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LiftCoefficients {
    /// Base lift coefficient (zero angle of attack).
    pub c_l_0: f64,
    /// Linear lift coefficient with respect to angle of attack (α).
    pub c_l_alpha: f64,
    /// Lift coefficient due to pitch rate (q).
    pub c_l_q: f64,
    /// Lift coefficient due to elevator deflection.
    pub c_l_deltae: f64,
    /// Combined lift coefficient due to angle of attack and pitch rate.
    pub c_l_alpha_q: f64,
    /// Quadratic lift coefficient with respect to α².
    pub c_l_alpha2: f64,
    /// Cubic lift coefficient with respect to α³.
    pub c_l_alpha3: f64,
    /// Quartic lift coefficient with respect to α⁴.
    pub c_l_alpha4: f64,
}

/// Side force coefficients for full aerodynamic model
#[derive(Component, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SideForceCoefficients {
    /// Side-force coefficient due to sideslip angle (β).
    pub c_y_beta: f64,
    /// Side-force coefficient due to roll rate (p).
    pub c_y_p: f64,
    /// Side-force coefficient due to yaw rate (r).
    pub c_y_r: f64,
    /// Side-force coefficient due to aileron deflection.
    pub c_y_deltaa: f64,
    /// Side-force coefficient due to rudder deflection.
    pub c_y_deltar: f64,
}

/// Roll coefficients for full aerodynamic model
#[derive(Component, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RollCoefficients {
    /// Roll moment coefficient due to sideslip angle (β).
    pub c_l_beta: f64,
    /// Roll moment coefficient due to roll rate (p).
    pub c_l_p: f64,
    /// Roll moment coefficient due to yaw rate (r).
    pub c_l_r: f64,
    /// Roll moment coefficient due to aileron deflection.
    pub c_l_deltaa: f64,
    /// Roll moment coefficient due to rudder deflection.
    pub c_l_deltar: f64,
}

/// Pitch coefficients for full aerodynamic model
#[derive(Component, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PitchCoefficients {
    /// Base pitch moment coefficient (zero angle of attack).
    pub c_m_0: f64,
    /// Linear pitch moment coefficient with respect to angle of attack (α).
    pub c_m_alpha: f64,
    /// Pitch moment coefficient due to pitch rate (q).
    pub c_m_q: f64,
    /// Pitch moment coefficient due to elevator deflection.
    pub c_m_deltae: f64,
    /// Combined pitch moment coefficient due to α and q.
    pub c_m_alpha_q: f64,
    /// Quadratic pitch moment coefficient due to α² and q.
    pub c_m_alpha2_q: f64,
    /// Quadratic pitch moment coefficient due to α² and elevator deflection.
    pub c_m_alpha2_deltae: f64,
    /// Cubic pitch moment coefficient due to α³ and q.
    pub c_m_alpha3_q: f64,
    /// Cubic pitch moment coefficient due to α³ and elevator deflection.
    pub c_m_alpha3_deltae: f64,
    /// Quartic pitch moment coefficient with respect to α⁴.
    pub c_m_alpha4: f64,
}

/// Yaw coefficients for full aerodynamic model
#[derive(Component, Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct YawCoefficients {
    /// Yaw moment coefficient due to sideslip angle (β).
    pub c_n_beta: f64,
    /// Yaw moment coefficient due to roll rate (p).
    pub c_n_p: f64,
    /// Yaw moment coefficient due to yaw rate (r).
    pub c_n_r: f64,
    /// Yaw moment coefficient due to aileron deflection.
    pub c_n_deltaa: f64,
    /// Yaw moment coefficient due to rudder deflection.
    pub c_n_deltar: f64,
    /// Quadratic yaw moment coefficient due to β².
    pub c_n_beta2: f64,
    /// Cubic yaw moment coefficient due to β³.
    pub c_n_beta3: f64,
}

impl AircraftAeroCoefficients {
    pub fn new(
        drag: DragCoefficients,
        lift: LiftCoefficients,
        side_force: SideForceCoefficients,
        roll: RollCoefficients,
        pitch: PitchCoefficients,
        yaw: YawCoefficients,
    ) -> Self {
        AircraftAeroCoefficients {
            drag,
            lift,
            side_force,
            roll,
            pitch,
            yaw,
        }
    }

    pub fn twin_otter() -> AircraftAeroCoefficients {
        AircraftAeroCoefficients::new(
            DragCoefficients::twin_otter(),
            LiftCoefficients::twin_otter(),
            SideForceCoefficients::twin_otter(),
            RollCoefficients::twin_otter(),
            PitchCoefficients::twin_otter(),
            YawCoefficients::twin_otter(),
        )
    }

    pub fn f4_phantom() -> AircraftAeroCoefficients {
        AircraftAeroCoefficients::new(
            DragCoefficients::f4_phantom(),
            LiftCoefficients::f4_phantom(),
            SideForceCoefficients::f4_phantom(),
            RollCoefficients::f4_phantom(),
            PitchCoefficients::f4_phantom(),
            YawCoefficients::f4_phantom(),
        )
    }

    pub fn generic_transport() -> Self {
        AircraftAeroCoefficients::new(
            DragCoefficients::generic_transport(),
            LiftCoefficients::generic_transport(),
            SideForceCoefficients::generic_transport(),
            RollCoefficients::generic_transport(),
            PitchCoefficients::generic_transport(),
            YawCoefficients::generic_transport(),
        )
    }

    pub fn cessna_172() -> AircraftAeroCoefficients {
        AircraftAeroCoefficients::new(
            DragCoefficients::cessna_172(),
            LiftCoefficients::cessna_172(),
            SideForceCoefficients::cessna_172(),
            RollCoefficients::cessna_172(),
            PitchCoefficients::cessna_172(),
            YawCoefficients::cessna_172(),
        )
    }

    pub fn f16c() -> AircraftAeroCoefficients {
        AircraftAeroCoefficients::new(
            DragCoefficients::f16c(),
            LiftCoefficients::f16c(),
            SideForceCoefficients::f16c(),
            RollCoefficients::f16c(),
            PitchCoefficients::f16c(),
            YawCoefficients::f16c(),
        )
    }
}

impl DragCoefficients {
    pub fn twin_otter() -> DragCoefficients {
        DragCoefficients {
            c_d_0: 0.108,
            c_d_alpha: 0.138,
            c_d_alpha_q: -54.05,
            c_d_alpha_deltae: 0.111,
            c_d_alpha2: 2.988,
            c_d_alpha2_q: -302.1,
            c_d_alpha2_deltae: 0.156,
            c_d_alpha3: -7.743,
            c_d_alpha3_q: -218.8,
            c_d_alpha4: 11.77,
        }
    }

    pub fn f4_phantom() -> DragCoefficients {
        DragCoefficients {
            c_d_0: 0.031,
            c_d_alpha: 0.280,
            c_d_alpha_q: -11.98,
            c_d_alpha_deltae: 0.0,
            c_d_alpha2: -1.818,
            c_d_alpha2_q: 209.4,
            c_d_alpha2_deltae: 0.515,
            c_d_alpha3: 22.27,
            c_d_alpha3_q: -284.7,
            c_d_alpha4: -29.81,
        }
    }

    pub fn generic_transport() -> DragCoefficients {
        DragCoefficients {
            c_d_0: 0.019,
            c_d_alpha: -0.078,
            c_d_alpha_q: -27.42,
            c_d_alpha_deltae: 0.293,
            c_d_alpha2: 3.420,
            c_d_alpha2_q: 288.2,
            c_d_alpha2_deltae: -0.040,
            c_d_alpha3: 1.819,
            c_d_alpha3_q: 355.3,
            c_d_alpha4: -6.563,
        }
    }

    pub fn cessna_172() -> DragCoefficients {
        DragCoefficients {
            c_d_0: 0.05,
            c_d_alpha: 0.0,
            c_d_alpha_q: 0.0,
            c_d_alpha_deltae: 0.0,
            c_d_alpha2: 0.0,
            c_d_alpha2_q: 0.0,
            c_d_alpha2_deltae: 0.0,
            c_d_alpha3: 0.0,
            c_d_alpha3_q: 0.0,
            c_d_alpha4: 0.0,
        }
    }

    pub fn f16c() -> DragCoefficients {
        DragCoefficients {
            c_d_0: 0.034,
            c_d_alpha: -0.005,
            c_d_alpha_q: 20.77,
            c_d_alpha_deltae: 0.177,
            c_d_alpha2: 1.285,
            c_d_alpha2_q: -19.97,
            c_d_alpha2_deltae: 0.756,
            c_d_alpha3: 5.887,
            c_d_alpha3_q: 55.59,
            c_d_alpha4: -5.155,
        }
    }
}

impl LiftCoefficients {
    pub fn twin_otter() -> LiftCoefficients {
        LiftCoefficients {
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

    pub fn f4_phantom() -> LiftCoefficients {
        LiftCoefficients {
            c_l_0: 0.105,
            c_l_alpha: 1.519,
            c_l_q: 6.727,
            c_l_deltae: 0.265,
            c_l_alpha_q: 33.25,
            c_l_alpha2: 9.90,
            c_l_alpha3: -12.71,
            c_l_alpha4: -12.91,
        }
    }

    pub fn generic_transport() -> LiftCoefficients {
        LiftCoefficients {
            c_l_0: 0.016,
            c_l_alpha: 5.343,
            c_l_q: 30.78,
            c_l_deltae: 0.396,
            c_l_alpha_q: 12.03,
            c_l_alpha2: 0.506,
            c_l_alpha3: -36.30,
            c_l_alpha4: 46.13,
        }
    }

    pub fn cessna_172() -> LiftCoefficients {
        LiftCoefficients {
            c_l_0: 0.3,
            c_l_alpha: 4.0,
            c_l_q: 0.0,
            c_l_deltae: 0.0,
            c_l_alpha_q: 0.0,
            c_l_alpha2: 0.0,
            c_l_alpha3: 0.0,
            c_l_alpha4: 0.0,
        }
    }

    pub fn f16c() -> LiftCoefficients {
        LiftCoefficients {
            c_l_0: 0.074,
            c_l_alpha: 4.458,
            c_l_q: 29.90,
            c_l_deltae: 0.412,
            c_l_alpha_q: -5.538,
            c_l_alpha2: -2.477,
            c_l_alpha3: -1.101,
            c_l_alpha4: 1.906,
        }
    }
}

impl SideForceCoefficients {
    pub fn twin_otter() -> SideForceCoefficients {
        SideForceCoefficients {
            c_y_beta: -0.885,
            c_y_p: -0.090,
            c_y_r: 1.697,
            c_y_deltaa: -0.051,
            c_y_deltar: -0.193,
        }
    }

    pub fn f4_phantom() -> SideForceCoefficients {
        SideForceCoefficients {
            c_y_beta: -0.688,
            c_y_p: 0.129,
            c_y_r: 0.670,
            c_y_deltaa: 0.0,
            c_y_deltar: 0.089,
        }
    }

    pub fn generic_transport() -> SideForceCoefficients {
        SideForceCoefficients {
            c_y_beta: -1.003,
            c_y_p: 0.033,
            c_y_r: 0.952,
            c_y_deltaa: -0.009,
            c_y_deltar: 0.253,
        }
    }

    pub fn cessna_172() -> SideForceCoefficients {
        SideForceCoefficients {
            c_y_beta: -0.5,
            c_y_p: 0.0,
            c_y_r: 0.0,
            c_y_deltaa: -0.0,
            c_y_deltar: 0.0,
        }
    }

    pub fn f16c() -> SideForceCoefficients {
        SideForceCoefficients {
            c_y_beta: -1.146,
            c_y_p: -0.188,
            c_y_r: 0.876,
            c_y_deltaa: 0.060,
            c_y_deltar: 0.164,
        }
    }
}

impl RollCoefficients {
    pub fn twin_otter() -> RollCoefficients {
        RollCoefficients {
            c_l_beta: -0.112,
            c_l_p: -0.413,
            c_l_r: 0.191,
            c_l_deltaa: 0.206,
            c_l_deltar: 0.116,
        }
    }

    pub fn f4_phantom() -> RollCoefficients {
        RollCoefficients {
            c_l_beta: -0.034,
            c_l_p: -0.236,
            c_l_r: 0.025,
            c_l_deltaa: 0.035,
            c_l_deltar: 0.013,
        }
    }

    pub fn generic_transport() -> RollCoefficients {
        RollCoefficients {
            c_l_beta: -0.109,
            c_l_p: -0.366,
            c_l_r: 0.061,
            c_l_deltaa: 0.079,
            c_l_deltar: 0.021,
        }
    }

    pub fn cessna_172() -> RollCoefficients {
        RollCoefficients {
            c_l_beta: -0.1,
            c_l_p: -0.5,
            c_l_r: 0.0,
            c_l_deltaa: 0.2,
            c_l_deltar: 0.0,
        }
    }

    pub fn f16c() -> RollCoefficients {
        RollCoefficients {
            c_l_beta: -0.071,
            c_l_p: -0.445,
            c_l_r: 0.058,
            c_l_deltaa: -0.143,
            c_l_deltar: 0.023,
        }
    }
}

impl PitchCoefficients {
    pub fn twin_otter() -> PitchCoefficients {
        PitchCoefficients {
            c_m_0: 0.057,
            c_m_alpha: -1.419,
            c_m_q: -27.95,
            c_m_deltae: 1.626,
            c_m_alpha_q: 100.7,
            c_m_alpha2_q: -759.2,
            c_m_alpha2_deltae: 7.664,
            c_m_alpha3_q: 1103.0,
            c_m_alpha3_deltae: -8.121,
            c_m_alpha4: 2.468,
        }
    }

    pub fn f4_phantom() -> PitchCoefficients {
        PitchCoefficients {
            c_m_0: -0.013,
            c_m_alpha: -0.254,
            c_m_q: -2.916,
            c_m_deltae: 0.403,
            c_m_alpha_q: -3.955,
            c_m_alpha2_q: -24.0,
            c_m_alpha2_deltae: -0.270,
            c_m_alpha3_q: 55.32,
            c_m_alpha3_deltae: 1.479,
            c_m_alpha4: -0.448,
        }
    }

    pub fn generic_transport() -> PitchCoefficients {
        PitchCoefficients {
            c_m_0: 0.182,
            c_m_alpha: -1.782,
            c_m_q: -44.34,
            c_m_deltae: 1.785,
            c_m_alpha_q: 374.0,
            c_m_alpha2_q: -1748.0,
            c_m_alpha2_deltae: 2.439,
            c_m_alpha3_q: 1949.0,
            c_m_alpha3_deltae: -0.038,
            c_m_alpha4: 0.803,
        }
    }

    pub fn cessna_172() -> PitchCoefficients {
        PitchCoefficients {
            c_m_0: 0.0,
            c_m_alpha: 2.0,
            c_m_q: 0.0,
            c_m_deltae: -2.0,
            c_m_alpha_q: 0.0,
            c_m_alpha2_q: 0.0,
            c_m_alpha2_deltae: 0.0,
            c_m_alpha3_q: 0.0,
            c_m_alpha3_deltae: 0.0,
            c_m_alpha4: 0.0,
        }
    }

    pub fn f16c() -> PitchCoefficients {
        PitchCoefficients {
            c_m_0: -0.024,
            c_m_alpha: -0.288,
            c_m_q: -8.267,
            c_m_deltae: -0.174,
            c_m_alpha_q: -4.315,
            c_m_alpha2_q: 15.47,
            c_m_alpha2_deltae: -1.057,
            c_m_alpha3_q: -18.25,
            c_m_alpha3_deltae: 0.848,
            c_m_alpha4: 0.581,
        }
    }
}

impl YawCoefficients {
    pub fn twin_otter() -> YawCoefficients {
        YawCoefficients {
            c_n_beta: 0.088,
            c_n_p: -0.043,
            c_n_r: -0.426,
            c_n_deltaa: 0.023,
            c_n_deltar: -0.087,
            c_n_beta2: 0.337,
            c_n_beta3: -0.766,
        }
    }

    pub fn f4_phantom() -> YawCoefficients {
        YawCoefficients {
            c_n_beta: 0.142,
            c_n_p: -0.006,
            c_n_r: -0.358,
            c_n_deltaa: 0.001,
            c_n_deltar: 0.053,
            c_n_beta2: 0.0,
            c_n_beta3: 0.377,
        }
    }

    pub fn generic_transport() -> YawCoefficients {
        YawCoefficients {
            c_n_beta: 0.183,
            c_n_p: -0.022,
            c_n_r: -0.405,
            c_n_deltaa: -0.009,
            c_n_deltar: 0.129,
            c_n_beta2: 0.184,
            c_n_beta3: -0.377,
        }
    }

    pub fn cessna_172() -> YawCoefficients {
        YawCoefficients {
            c_n_beta: 0.1,
            c_n_p: 0.0,
            c_n_r: -0.15,
            c_n_deltaa: 0.0,
            c_n_deltar: 0.0,
            c_n_beta2: 0.0,
            c_n_beta3: 0.0,
        }
    }

    pub fn f16c() -> YawCoefficients {
        YawCoefficients {
            c_n_beta: 0.102,
            c_n_p: 0.056,
            c_n_r: -0.282,
            c_n_deltaa: -0.014,
            c_n_deltar: -0.046,
            c_n_beta2: 0.086,
            c_n_beta3: -0.140,
        }
    }
}
