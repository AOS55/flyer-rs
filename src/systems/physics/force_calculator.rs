use crate::components::{Force, ForceCategory, PhysicsComponent, ReferenceFrame, SpatialComponent};
use crate::resources::PhysicsConfig;
use bevy::prelude::*;

/// A system to calculate the net forces and moments acting on entities with a `PhysicsComponent`.
/// It processes:
/// - Gravitational force.
/// - Forces and moments applied in different reference frames (Body, Inertial, Wind).
/// The results are stored in the `net_force` and `net_moment` fields of the `PhysicsComponent`.
///
/// # Arguments
/// - `query`: Query to access entities with FullAircraftState.
/// - `config`: A resource containing global physics configuration parameters, like gravity.
pub fn force_calculator_system(
    mut query: Query<(&mut PhysicsComponent, &SpatialComponent)>,
    config: Res<PhysicsConfig>,
) {
    for (mut physics, spatial) in query.iter_mut() {
        // Reset net forces before calculation
        physics.net_force = nalgebra::Vector3::zeros();
        physics.net_moment = nalgebra::Vector3::zeros();

        // Remove old gravitational forces but keep others
        physics
            .forces
            .retain(|force| force.category != ForceCategory::Gravitational);
        physics
            .moments
            .retain(|moment| moment.category != ForceCategory::Gravitational);

        // Clone forces for iteration to avoid borrow conflict
        let forces = physics.forces.clone();
        let moments = physics.moments.clone();

        // Process forces - but don't store transformed versions
        for force in forces.iter() {
            // Transform force to inertial frame
            let force_inertial = match force.frame {
                ReferenceFrame::Body => spatial.attitude * force.vector,
                ReferenceFrame::Inertial => force.vector,
                ReferenceFrame::Wind => spatial.attitude * force.vector,
            };

            // Add to net force
            physics.net_force += force_inertial;

            // Calculate moment if force has application point
            if let Some(point) = force.point {
                let point_inertial = spatial.attitude * point;
                let moment = point_inertial.cross(&force_inertial);
                physics.net_moment += moment;
            }
        }

        // Process moments - but keep them in their original frame
        for moment in moments.iter() {
            let moment_inertial = match moment.frame {
                ReferenceFrame::Body => spatial.attitude * moment.vector,
                ReferenceFrame::Inertial => moment.vector,
                ReferenceFrame::Wind => spatial.attitude * moment.vector,
            };
            physics.net_moment += moment_inertial;
        }

        // Add gravitational force
        let gravity_force = Force {
            vector: config.gravity * physics.mass,
            point: None,
            frame: ReferenceFrame::Inertial,
            category: ForceCategory::Gravitational,
        };
        physics.net_force += gravity_force.vector;
        physics.forces.push(gravity_force.clone()); // Use clone to avoid move
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{UnitQuaternion, Vector3};
    use std::f64::consts::PI;

    fn setup_test_world() -> (World, PhysicsConfig) {
        let world = World::new();
        let physics_config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        };
        (world, physics_config)
    }

    #[test]
    fn test_basic_force_calculation() {
        // Setup
        let (mut world, physics_config) = setup_test_world();
        world.insert_resource(physics_config.clone());

        let mass = 100.0; // 100kg
        let mut physics = PhysicsComponent::new(mass, nalgebra::Matrix3::identity() * 10.0);

        // Add a test force in body frame
        let test_force = Force {
            vector: Vector3::new(10.0, 0.0, 0.0), // 10N forward in body frame
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        };
        physics.add_force(test_force);

        // Create spatial component with 45 degree pitch up
        let spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::from_euler_angles(0.0, PI / 4.0, 0.0),
            angular_velocity: Vector3::zeros(),
        };

        // Spawn entity
        let _entity = world.spawn((physics, spatial)).id();

        // Run system
        println!("Attitude quaternion: {:?}", spatial.attitude);

        // Before running the system, manually calculate the transformation
        let body_force = Vector3::new(10.0, 0.0, 0.0);
        let rotated_force = spatial.attitude * body_force;
        println!("Manual force transformation:");
        println!("  Body force: {:?}", body_force);
        println!("  Rotated force: {:?}", rotated_force);

        // Run system and get results
        let mut schedule = Schedule::default();
        schedule.add_systems(force_calculator_system);
        schedule.run(&mut world);

        let physics = world.query::<&PhysicsComponent>().single(&world);

        println!("\nFinal forces:");
        println!(
            "  Net force before gravity: {:?}",
            physics.net_force - physics_config.gravity * mass
        );
        println!(
            "  Gravity contribution: {:?}",
            physics_config.gravity * mass
        );
        println!("  Total net force: {:?}", physics.net_force);

        // For a 45-degree pitch up:
        let sqrt2_2 = (2.0_f64).sqrt() / 2.0; // cos(45°) = sin(45°) = 1/√2
        let force_magnitude = 10.0;

        // Forward force decomposition
        let expected_force_x = force_magnitude * sqrt2_2; // Forward component

        // Vertical force decomposition plus gravity
        let expected_force_z = -force_magnitude * sqrt2_2 + mass * physics_config.gravity.z;

        println!("Forces:");
        println!("  Expected X: {}", expected_force_x);
        println!("  Actual X: {}", physics.net_force.x);
        println!("  Expected Z: {}", expected_force_z);
        println!("  Actual Z: {}", physics.net_force.z);

        // Check forces with reasonable tolerance
        let tolerance = 1e-10;
        assert!(
            (physics.net_force.x - expected_force_x).abs() < tolerance,
            "X force mismatch: got {}, expected {}",
            physics.net_force.x,
            expected_force_x
        );
        assert!(
            (physics.net_force.z - expected_force_z).abs() < tolerance,
            "Z force mismatch: got {}, expected {}",
            physics.net_force.z,
            expected_force_z
        );
        assert!(
            physics.net_force.y.abs() < tolerance,
            "Y force should be zero, got {}",
            physics.net_force.y
        );
    }

    #[test]
    fn test_moment_calculation() {
        let (mut world, physics_config) = setup_test_world();
        world.insert_resource(physics_config.clone());

        let mass = 100.0;
        let mut physics = PhysicsComponent::new(mass, nalgebra::Matrix3::identity() * 10.0);

        // Add a force with a moment arm
        let test_force = Force {
            vector: Vector3::new(0.0, 0.0, -10.0), // 10N downward in body frame
            point: Some(Vector3::new(1.0, 0.0, 0.0)), // 1m forward of CG
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        };
        physics.add_force(test_force);

        let spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        let _entity = world.spawn((physics, spatial)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(force_calculator_system);
        schedule.run(&mut world);

        let physics = world.query::<&PhysicsComponent>().single(&world);

        // Expected: 10N * 1m moment arm = 10Nm pitch up moment
        let expected_moment_y = 10.0;
        let tolerance = 1e-10;

        assert!(
            (physics.net_moment.y - expected_moment_y).abs() < tolerance,
            "Pitch moment mismatch: got {}, expected {}",
            physics.net_moment.y,
            expected_moment_y
        );
        assert!(
            physics.net_moment.x.abs() < tolerance,
            "Roll moment should be zero"
        );
        assert!(
            physics.net_moment.z.abs() < tolerance,
            "Yaw moment should be zero"
        );
    }
}
