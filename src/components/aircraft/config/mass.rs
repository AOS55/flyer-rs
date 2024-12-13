use bevy::prelude::*;
use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MassModel {
    pub mass: f64,
    pub inertia: Matrix3<f64>,
    pub inertia_inv: Matrix3<f64>,
}

impl MassModel {
    pub fn new(mass: f64, ixx: f64, iyy: f64, izz: f64, ixz: f64) -> Self {
        let inertia = Matrix3::from_columns(&[
            Vector3::new(ixx, 0.0, -ixz),
            Vector3::new(0.0, iyy, 0.0),
            Vector3::new(-ixz, 0.0, izz),
        ]);
        let inertia_inv = inertia.try_inverse().unwrap_or_else(|| {
            error!("Warning: Inertia matrix is uninvertable, defaulting to zero matrix.");
            Matrix3::zeros() // Default to a zero matrix if uninvertable
        });

        Self {
            mass,
            inertia,
            inertia_inv,
        }
    }

    pub fn twin_otter() -> Self {
        Self::new(4874.8, 28366.4, 32852.8, 52097.3, 1384.3)
    }

    pub fn f4_phantom() -> Self {
        Self::new(17642.0, 33898.0, 165669.0, 189496.0, 2952.0)
    }

    pub fn generic_transport() -> Self {
        Self::new(22.5, 67.2, 5.77, 7.39, 0.163)
    }
}
