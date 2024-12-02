use crate::components::{PhysicsComponent, ReferenceFrame, SpatialComponent};
use crate::ecs::error::Result;
use crate::ecs::{System, World};
use nalgebra::Vector3;

pub struct ForceCalculator {
    gravity: Vector3<f64>,
}

impl ForceCalculator {
    pub fn new() -> Self {
        Self {
            gravity: Vector3::new(0.0, 0.0, -9.81),
        }
    }

    fn calculate_forces(&self, physics: &mut PhysicsComponent, spatial: &SpatialComponent) {
        // Reset net forces and moments
        physics.net_force = Vector3::zeros();
        physics.net_moment = Vector3::zeros();

        // Add gravitational force (always in inertial frame)
        let gravity_force = self.gravity * physics.mass;
        physics.net_force += gravity_force;

        // Process all forces
        for force in &physics.forces {
            let force_inertial = match force.frame {
                ReferenceFrame::Body => spatial.attitude * force.vector,
                ReferenceFrame::Inertial => force.vector,
                ReferenceFrame::Wind => {
                    // For now, treat wind frame same as body frame
                    // This should be updated with proper wind calculations
                    spatial.attitude * force.vector
                }
            };

            physics.net_force += force_inertial;

            // Calculate moment if point of application is specified
            if let Some(point) = force.point {
                let point_inertial = spatial.attitude * point;
                let moment = point_inertial.cross(&force_inertial);
                physics.net_moment += moment;
            }
        }

        // Process all moments
        for moment in &physics.moments {
            let moment_inertial = match moment.frame {
                ReferenceFrame::Body => spatial.attitude * moment.vector,
                ReferenceFrame::Inertial => moment.vector,
                ReferenceFrame::Wind => spatial.attitude * moment.vector,
            };
            physics.net_moment += moment_inertial;
        }
    }
}

impl System for ForceCalculator {
    fn name(&self) -> &str {
        "Force Calculator"
    }

    fn run(&mut self, world: &mut World) -> Result<()> {
        let mut spatial_data = Vec::new();

        // First collect all spatial components
        for (entity, spatial) in world.query::<SpatialComponent>() {
            spatial_data.push((entity, spatial.clone()));
        }

        // Then update physics components
        for (entity, spatial) in spatial_data {
            if let Ok(physics) = world.get_component_mut::<PhysicsComponent>(entity) {
                self.calculate_forces(physics, &spatial);
            }
        }

        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Moment;
    use crate::components::{Force, ForceCategory};
    use approx::assert_relative_eq;
    use nalgebra::{UnitQuaternion, Vector3};

    // Helper function to create a basic physics component
    fn create_test_physics() -> PhysicsComponent {
        PhysicsComponent::new(
            1.0,                           // mass
            nalgebra::Matrix3::identity(), // inertia
        )
    }

    // Helper function to create a basic spatial component
    fn create_test_spatial() -> SpatialComponent {
        SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        }
    }

    #[test]
    fn test_gravity_force() {
        let calculator = ForceCalculator::new();
        let mut physics = create_test_physics();
        let spatial = create_test_spatial();

        // Calculate forces
        calculator.calculate_forces(&mut physics, &spatial);

        // Check if gravity force is correct (mass * gravity)
        assert_relative_eq!(physics.net_force.z, -9.81, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.x, 0.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.y, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_body_force_transformation() {
        let calculator = ForceCalculator::new();
        let mut physics = create_test_physics();
        let mut spatial = create_test_spatial();

        // Set up a 90-degree rotation around Y axis
        spatial.attitude =
            UnitQuaternion::from_axis_angle(&Vector3::y_axis(), std::f64::consts::FRAC_PI_2);

        // Add a force in body frame (pointing forward in body frame)
        physics.add_force(Force {
            vector: Vector3::new(10.0, 0.0, 0.0),
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Custom("test".to_string()),
        });

        calculator.calculate_forces(&mut physics, &spatial);

        // After 90-degree Y rotation, body X axis aligns with inertial -Z axis
        // Net force should include both gravity and transformed body force
        assert_relative_eq!(physics.net_force.x, 0.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.y, 0.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.z, -9.81 - 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_force_with_moment() {
        let calculator = ForceCalculator::new();
        let mut physics = create_test_physics();
        let spatial = create_test_spatial();

        // Add a force with an offset point (should create a moment)
        physics.add_force(Force {
            vector: Vector3::new(1.0, 0.0, 0.0),
            point: Some(Vector3::new(0.0, 1.0, 0.0)),
            frame: ReferenceFrame::Body,
            category: ForceCategory::Custom("test".to_string()),
        });

        calculator.calculate_forces(&mut physics, &spatial);
        assert_relative_eq!(physics.net_moment.z, -1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_multiple_forces() {
        let calculator = ForceCalculator::new();
        let mut physics = create_test_physics();
        let spatial = create_test_spatial();

        // Add multiple forces in different frames
        physics.add_force(Force {
            vector: Vector3::new(1.0, 0.0, 0.0),
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Custom("force1".to_string()),
        });

        physics.add_force(Force {
            vector: Vector3::new(0.0, 2.0, 0.0),
            point: None,
            frame: ReferenceFrame::Inertial,
            category: ForceCategory::Custom("force2".to_string()),
        });

        calculator.calculate_forces(&mut physics, &spatial);

        // Check net force (including gravity)
        assert_relative_eq!(physics.net_force.x, 1.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.y, 2.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.z, -9.81, epsilon = 1e-10);
    }

    #[test]
    fn test_force_categories() {
        let calculator = ForceCalculator::new();
        let mut physics = create_test_physics();
        let spatial = create_test_spatial();

        // Add forces with different categories
        physics.add_force(Force {
            vector: Vector3::new(1.0, 0.0, 0.0),
            point: None,
            frame: ReferenceFrame::Inertial,
            category: ForceCategory::Aerodynamic,
        });

        physics.add_force(Force {
            vector: Vector3::new(0.0, 1.0, 0.0),
            point: None,
            frame: ReferenceFrame::Inertial,
            category: ForceCategory::Propulsive,
        });

        calculator.calculate_forces(&mut physics, &spatial);

        // Verify forces are accumulated correctly
        assert_relative_eq!(physics.net_force.x, 1.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.y, 1.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.z, -9.81, epsilon = 1e-10);
    }

    #[test]
    fn test_moment_accumulation() {
        let calculator = ForceCalculator::new();
        let mut physics = create_test_physics();
        let spatial = create_test_spatial();

        // Add direct moments in different frames
        physics.moments.push(Moment {
            vector: Vector3::new(1.0, 0.0, 0.0),
            frame: ReferenceFrame::Body,
            category: ForceCategory::Custom("moment1".to_string()),
        });

        physics.moments.push(Moment {
            vector: Vector3::new(0.0, 2.0, 0.0),
            frame: ReferenceFrame::Inertial,
            category: ForceCategory::Custom("moment2".to_string()),
        });

        calculator.calculate_forces(&mut physics, &spatial);

        // Check accumulated moments
        assert_relative_eq!(physics.net_moment.x, 1.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_moment.y, 2.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_moment.z, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_force_clearing() {
        let calculator = ForceCalculator::new();
        let mut physics = create_test_physics();
        let spatial = create_test_spatial();

        // Add some forces
        physics.add_force(Force {
            vector: Vector3::new(1.0, 0.0, 0.0),
            point: None,
            frame: ReferenceFrame::Inertial,
            category: ForceCategory::Custom("test".to_string()),
        });

        // Clear forces
        physics.clear_forces();

        calculator.calculate_forces(&mut physics, &spatial);

        // Only gravity should remain
        assert_relative_eq!(physics.net_force.x, 0.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.y, 0.0, epsilon = 1e-10);
        assert_relative_eq!(physics.net_force.z, -9.81, epsilon = 1e-10);
    }
}
