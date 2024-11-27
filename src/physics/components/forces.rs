use nalgebra::{Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents different types of forces that can act on a body
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForceType {
    Aerodynamic,
    Propulsive,
    Gravitational,
    Contact,
    Ground,
    Custom(String),
}

/// Represents a force vector with application point and type
#[derive(Debug, Clone)]
pub struct Force {
    /// Force vector in Newtons
    pub magnitude: Vector3<f64>,
    /// Point of application in body frame coordinates (if None, force is applied at CG)
    pub application_point: Option<Point3<f64>>,
    /// Reference frame the force is expressed in
    pub frame: ReferenceFrame,
    /// Type of force
    pub force_type: ForceType,
    /// Unique identifier for the force
    pub id: ForceId,
}

/// Represents a moment/torque vector
#[derive(Debug, Clone)]
pub struct Moment {
    /// Moment vector in Newton-meters
    pub magnitude: Vector3<f64>,
    /// Reference frame the moment is expressed in
    pub frame: ReferenceFrame,
    /// Type of moment
    pub force_type: ForceType,
    /// Unique identifier for the moment
    pub id: ForceId,
}

/// Reference frames for forces and moments
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferenceFrame {
    /// Inertial/Earth-fixed frame
    Inertial,
    /// Body-fixed frame
    Body,
    /// Wind frame
    Wind,
}

/// Force identifier type
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ForceId(String);

impl ForceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Complete force and moment system for a body
#[derive(Debug, Clone)]
pub struct ForceSystem {
    /// All forces acting on the body
    forces: HashMap<ForceId, Force>,
    /// All moments acting on the body
    moments: HashMap<ForceId, Moment>,
    /// Body attitude for frame transformations
    attitude: UnitQuaternion<f64>,
}

impl Default for ForceSystem {
    fn default() -> Self {
        Self {
            forces: HashMap::new(),
            moments: HashMap::new(),
            attitude: UnitQuaternion::identity(),
        }
    }
}

impl ForceSystem {
    pub fn new() -> Self {
        Self {
            forces: HashMap::new(),
            moments: HashMap::new(),
            attitude: UnitQuaternion::identity(),
        }
    }

    /// Set the current attitude for frame transformations
    pub fn set_attitude(&mut self, attitude: UnitQuaternion<f64>) {
        self.attitude = attitude;
    }

    /// Add or update a force
    pub fn add_force(&mut self, force: Force) {
        self.forces.insert(force.id.clone(), force);
    }

    /// Add or update a moment
    pub fn add_moment(&mut self, moment: Moment) {
        self.moments.insert(moment.id.clone(), moment);
    }

    /// Remove a force by ID
    pub fn remove_force(&mut self, id: &ForceId) -> Option<Force> {
        self.forces.remove(id)
    }

    /// Remove a moment by ID
    pub fn remove_moment(&mut self, id: &ForceId) -> Option<Moment> {
        self.moments.remove(id)
    }

    /// Clear all forces and moments
    pub fn clear(&mut self) {
        self.forces.clear();
        self.moments.clear();
    }

    /// Get net force in inertial frame
    pub fn net_force(&self) -> Vector3<f64> {
        self.forces
            .values()
            .map(|force| self.transform_force_to_inertial(force))
            .sum()
    }

    /// Get net moment about CG in body frame
    pub fn net_moment(&self) -> Vector3<f64> {
        let force_moments = self
            .forces
            .values()
            .filter_map(|force| {
                force.application_point.map(|point| {
                    let force_in_body = self.transform_force_to_body(force);
                    point.coords.cross(&force_in_body)
                })
            })
            .sum::<Vector3<f64>>();

        let direct_moments = self
            .moments
            .values()
            .map(|moment| self.transform_moment_to_body(moment))
            .sum::<Vector3<f64>>();

        force_moments + direct_moments
    }

    /// Get forces of a specific type in inertial frame
    pub fn get_forces_by_type(&self, force_type: ForceType) -> Vector3<f64> {
        self.forces
            .values()
            .filter(|f| f.force_type == force_type)
            .map(|force| self.transform_force_to_inertial(force))
            .sum()
    }

    /// Get moments of a specific type in body frame
    pub fn get_moments_by_type(&self, force_type: ForceType) -> Vector3<f64> {
        let force_moments = self
            .forces
            .values()
            .filter(|f| f.force_type == force_type)
            .filter_map(|force| {
                force.application_point.map(|point| {
                    let force_in_body = self.transform_force_to_body(force);
                    point.coords.cross(&force_in_body)
                })
            })
            .sum::<Vector3<f64>>();

        let direct_moments = self
            .moments
            .values()
            .filter(|m| m.force_type == force_type)
            .map(|moment| self.transform_moment_to_body(moment))
            .sum::<Vector3<f64>>();

        force_moments + direct_moments
    }

    /// Transform force to inertial frame
    fn transform_force_to_inertial(&self, force: &Force) -> Vector3<f64> {
        match force.frame {
            ReferenceFrame::Inertial => force.magnitude,
            ReferenceFrame::Body => self.attitude * force.magnitude,
            ReferenceFrame::Wind => {
                // For wind frame, we would need wind direction information
                // This is a simplified transformation
                self.attitude * force.magnitude
            }
        }
    }

    /// Transform force to body frame
    fn transform_force_to_body(&self, force: &Force) -> Vector3<f64> {
        match force.frame {
            ReferenceFrame::Inertial => self.attitude.inverse() * force.magnitude,
            ReferenceFrame::Body => force.magnitude,
            ReferenceFrame::Wind => {
                // Simplified wind to body transformation
                force.magnitude
            }
        }
    }

    /// Transform moment to body frame
    fn transform_moment_to_body(&self, moment: &Moment) -> Vector3<f64> {
        match moment.frame {
            ReferenceFrame::Inertial => self.attitude.inverse() * moment.magnitude,
            ReferenceFrame::Body => moment.magnitude,
            ReferenceFrame::Wind => {
                // Simplified wind to body transformation
                moment.magnitude
            }
        }
    }

    /// Get a force by ID
    pub fn get_force(&self, id: &ForceId) -> Option<&Force> {
        self.forces.get(id)
    }

    /// Get a moment by ID
    pub fn get_moment(&self, id: &ForceId) -> Option<&Moment> {
        self.moments.get(id)
    }

    /// Get all forces
    pub fn forces(&self) -> impl Iterator<Item = &Force> {
        self.forces.values()
    }

    /// Get all moments
    pub fn moments(&self) -> impl Iterator<Item = &Moment> {
        self.moments.values()
    }
}

impl Force {
    pub fn new(
        magnitude: Vector3<f64>,
        application_point: Option<Point3<f64>>,
        frame: ReferenceFrame,
        force_type: ForceType,
        id: impl Into<String>,
    ) -> Self {
        Self {
            magnitude,
            application_point,
            frame,
            force_type,
            id: ForceId::new(id),
        }
    }

    /// Create a force applied at the CG in body frame
    pub fn body_force(
        magnitude: Vector3<f64>,
        force_type: ForceType,
        id: impl Into<String>,
    ) -> Self {
        Self::new(magnitude, None, ReferenceFrame::Body, force_type, id)
    }

    /// Create a force in inertial frame
    pub fn inertial_force(
        magnitude: Vector3<f64>,
        force_type: ForceType,
        id: impl Into<String>,
    ) -> Self {
        Self::new(magnitude, None, ReferenceFrame::Inertial, force_type, id)
    }
}

impl Moment {
    pub fn new(
        magnitude: Vector3<f64>,
        frame: ReferenceFrame,
        force_type: ForceType,
        id: impl Into<String>,
    ) -> Self {
        Self {
            magnitude,
            frame,
            force_type,
            id: ForceId::new(id),
        }
    }

    /// Create a moment in body frame
    pub fn body_moment(
        magnitude: Vector3<f64>,
        force_type: ForceType,
        id: impl Into<String>,
    ) -> Self {
        Self::new(magnitude, ReferenceFrame::Body, force_type, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_force_system_basic() {
        let mut system = ForceSystem::new();

        // Add a simple force
        let force = Force::body_force(
            Vector3::new(1.0, 0.0, 0.0),
            ForceType::Aerodynamic,
            "test_force",
        );
        system.add_force(force);

        // Check net force
        let net = system.net_force();
        assert!((net.x - 1.0).abs() < 1e-10);
        assert!(net.y.abs() < 1e-10);
        assert!(net.z.abs() < 1e-10);
    }

    #[test]
    fn test_force_system_with_rotation() {
        let mut system = ForceSystem::new();

        // Set attitude (90 degree rotation around Z)
        system.set_attitude(UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.0));

        // Add force in body frame
        let force = Force::body_force(
            Vector3::new(1.0, 0.0, 0.0),
            ForceType::Aerodynamic,
            "rotated_force",
        );
        system.add_force(force);

        // Check net force in inertial frame
        let net = system.net_force();
        assert!((net.y - 1.0).abs() < 1e-10); // Force should now point in Y direction
        assert!(net.x.abs() < 1e-10);
        assert!(net.z.abs() < 1e-10);
    }

    #[test]
    fn test_moments() {
        let mut system = ForceSystem::new();

        // Add a force with moment arm
        let force = Force::new(
            Vector3::new(1.0, 0.0, 0.0),
            Some(Point3::new(0.0, 1.0, 0.0)),
            ReferenceFrame::Body,
            ForceType::Aerodynamic,
            "force_with_moment",
        );
        system.add_force(force);

        // Add a direct moment
        let moment = Moment::body_moment(
            Vector3::new(0.0, 0.0, 1.0),
            ForceType::Aerodynamic,
            "direct_moment",
        );
        system.add_moment(moment);

        // Check net moment
        let net_moment = system.net_moment();
        assert!(net_moment.x.abs() < 1e-10);
        assert!(net_moment.y.abs() < 1e-10);
        assert!((net_moment.z - 2.0).abs() < 1e-10); // 1 from force moment + 1 direct moment
    }

    #[test]
    fn test_force_types() {
        let mut system = ForceSystem::new();

        // Add forces of different types
        let aero_force =
            Force::body_force(Vector3::new(1.0, 0.0, 0.0), ForceType::Aerodynamic, "aero");
        let prop_force =
            Force::body_force(Vector3::new(2.0, 0.0, 0.0), ForceType::Propulsive, "prop");

        system.add_force(aero_force);
        system.add_force(prop_force);

        // Check forces by type
        let aero_forces = system.get_forces_by_type(ForceType::Aerodynamic);
        let prop_forces = system.get_forces_by_type(ForceType::Propulsive);

        assert!((aero_forces.x - 1.0).abs() < 1e-10);
        assert!((prop_forces.x - 2.0).abs() < 1e-10);
    }
}
