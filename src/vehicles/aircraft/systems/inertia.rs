use nalgebra::Matrix3;
use serde::{Deserialize, Serialize};

/// Aircraft Inertia data
#[derive(Debug, Deserialize, Serialize)]
pub struct Inertia {
    /// xx-axis inertia [Kg.m^2]
    ixx: f64,
    /// yy-axis inertia [Kg.m^2]
    iyy: f64,
    /// zz-axis inertia [Kg.m^2]
    izz: f64,
    /// xz-axis inertia [Kg.m^2]
    ixz: f64,
}

impl Inertia {
    /// Creates a 3x3 inertia matrix based upon the inertia structure
    pub const fn create_inertia(&self) -> Matrix3<f64> {
        Matrix3::new(
            self.ixx, 0.0, self.ixz, 0.0, self.iyy, 0.0, self.ixz, 0.0, self.izz,
        )
    }
}
