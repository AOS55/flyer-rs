use bevy::prelude::*;
use nalgebra::{Matrix3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsComponent {
    pub mass: f64,
    pub inertia: Matrix3<f64>,
    pub inertia_inv: Matrix3<f64>,

    // Source of truth for forces and moments
    pub forces: Vec<Force>,
    pub moments: Vec<Moment>,

    // Computed Results
    pub net_force: Vector3<f64>,
    pub net_moment: Vector3<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Force {
    pub vector: Vector3<f64>,
    pub point: Option<Vector3<f64>>,
    pub frame: ReferenceFrame,
    pub category: ForceCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Moment {
    pub vector: Vector3<f64>,
    pub frame: ReferenceFrame,
    pub category: ForceCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReferenceFrame {
    Body,
    Inertial,
    Wind,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ForceCategory {
    Aerodynamic,
    Propulsive,
    Gravitational,
    Ground,
    Custom,
}

impl PhysicsComponent {
    pub fn new(mass: f64, inertia: Matrix3<f64>) -> Self {
        let inertia_inv = inertia.try_inverse().unwrap_or(Matrix3::identity());
        Self {
            mass,
            inertia,
            inertia_inv,
            net_force: Vector3::zeros(),
            net_moment: Vector3::zeros(),
            forces: Vec::new(),
            moments: Vec::new(),
        }
    }

    pub fn add_force(&mut self, force: Force) {
        // Store the force for later processing
        self.forces.push(force);
    }

    pub fn add_moment(&mut self, moment: Moment) {
        // Store the moment for later processing
        self.moments.push(moment);
    }

    // Clear all forces and moments
    pub fn clear_forces(&mut self) {
        self.forces.clear();
        self.moments.clear();
        self.net_force = Vector3::zeros();
        self.net_moment = Vector3::zeros();
    }

    // Calculate net force (inertial frame) in a single location
    pub fn compute_net_forces(&mut self, attitude: &UnitQuaternion<f64>) {
        self.net_force = Vector3::zeros();
        self.net_moment = Vector3::zeros();

        for force in &self.forces {
            let force_inertial = match force.frame {
                ReferenceFrame::Body => attitude * force.vector,
                ReferenceFrame::Inertial => force.vector,
                ReferenceFrame::Wind => attitude * force.vector,
            };

            self.net_force += force_inertial;

            if let Some(point) = force.point {
                let point_inertial = attitude * point;
                self.net_moment += point_inertial.cross(&force_inertial);
            }
        }
    }
}
