use bevy::prelude::*;
use bevy::time::Time;
use nalgebra::{UnitQuaternion, Vector3};
use std::time::Duration;

use crate::{
    components::{
        AirData, AircraftControlSurfaces, FullAircraftConfig, PhysicsComponent, PropulsionState,
        SpatialComponent,
    },
    resources::{AerodynamicsConfig, EnvironmentConfig, EnvironmentModel, PhysicsConfig},
    systems::{
        aero_force_system, air_data_system, force_calculator_system, physics_integrator_system,
    },
};

// Define system sets for ordering
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhysicsSet {
    AirData,
    Forces,
    Integration,
}

/// A virtual physics simulation that can run independently of the main simulation time
pub struct VirtualPhysics {
    pub world: World,
    schedule: Schedule,
    dt: f64,
}

impl VirtualPhysics {
    pub fn new(physics_config: &PhysicsConfig) -> Self {
        let mut world = World::new();

        // Add required resources
        world.insert_resource(physics_config.clone());
        world.insert_resource(Time::<Fixed>::from_seconds(physics_config.timestep));

        let env_config = EnvironmentConfig::default(); // Assume nil wind for now
        world.insert_resource(EnvironmentModel::new(&env_config));
        world.insert_resource(AerodynamicsConfig {
            min_airspeed_threshold: 0.0,
        });

        // Create and configure schedule
        let mut schedule = Schedule::default();

        // Configure sets to ensure proper ordering
        schedule.configure_sets(
            (
                PhysicsSet::AirData,
                PhysicsSet::Forces,
                PhysicsSet::Integration,
            )
                .chain(),
        );

        // Add systems to their respective sets
        schedule.add_systems(air_data_system.in_set(PhysicsSet::AirData));
        schedule
            .add_systems((aero_force_system, force_calculator_system).in_set(PhysicsSet::Forces));
        schedule.add_systems(physics_integrator_system.in_set(PhysicsSet::Integration));

        Self {
            world,
            schedule,
            dt: physics_config.timestep,
        }
    }

    /// Create a virtual aircraft entity with given state and config
    pub fn spawn_aircraft(
        &mut self,
        spatial: &SpatialComponent,
        propulsion: &PropulsionState,
        config: &FullAircraftConfig,
    ) -> Entity {
        let air_data = AirData::default();
        let control_surfaces = AircraftControlSurfaces::default();
        let spatial = spatial.clone();
        let physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);
        let propulsion = propulsion.clone();
        self.world
            .spawn((
                air_data,
                control_surfaces,
                spatial,
                physics,
                propulsion,
                config.clone(),
            ))
            .id()
    }

    /// Run the physics simulation for a specified number of steps
    pub fn run_steps(&mut self, _entity: Entity, steps: usize) {
        for _ in 0..steps {
            // Update time resource
            if let Some(mut time) = self.world.get_resource_mut::<Time<Fixed>>() {
                time.advance_by(Duration::from_secs_f64(self.dt));
            }

            // Run all systems in sequence using the configured schedule
            self.schedule.run(&mut self.world);
        }
    }

    pub fn calculate_forces(&mut self, entity: Entity) -> (Vector3<f64>, Vector3<f64>) {
        // Reset forces at the start
        if let Some(mut physics) = self.world.get_mut::<PhysicsComponent>(entity) {
            physics.net_force = Vector3::zeros();
            physics.net_moment = Vector3::zeros();

            // Clear existing forces to ensure clean state
            physics.forces.clear();
            physics.moments.clear();
        }

        // Create a schedule for force calculation
        let mut force_schedule = Schedule::default();

        // Configure systems to run in sequence
        force_schedule.configure_sets((PhysicsSet::AirData, PhysicsSet::Forces).chain());

        // Add air data system
        force_schedule.add_systems(air_data_system.in_set(PhysicsSet::AirData));

        // Add force calculation systems
        force_schedule.add_systems(
            (aero_force_system, force_calculator_system)
                .chain()
                .in_set(PhysicsSet::Forces),
        );

        // Run the schedule
        force_schedule.run(&mut self.world);

        // Get the final forces and moments
        let physics = self
            .world
            .get::<PhysicsComponent>(entity)
            .expect("Entity should have PhysicsComponent");

        (physics.net_force, physics.net_moment)
    }

    /// Resets forces and moments on an entity
    pub fn reset_forces(&mut self, entity: Entity) {
        if let Some(mut physics) = self.world.get_mut::<PhysicsComponent>(entity) {
            physics.net_force = Vector3::zeros();
            physics.net_moment = Vector3::zeros();
            physics.forces.clear();
            physics.moments.clear();
        }
    }

    /// Get the current state of the virtual aircraft
    pub fn get_state(&self, entity: Entity) -> (SpatialComponent, AircraftControlSurfaces) {
        let spatial = self
            .world
            .get::<SpatialComponent>(entity)
            .expect("Entity should have SpatialComponent")
            .clone();

        let control_surfaces = self
            .world
            .get::<AircraftControlSurfaces>(entity)
            .expect("Entity should have AircraftControlSurfaces")
            .clone();

        (spatial, control_surfaces)
    }

    /// Set the state of the virtual aircraft
    pub fn set_state(
        &mut self,
        entity: Entity,
        velocity: &Vector3<f64>,
        attitude: &UnitQuaternion<f64>,
    ) {
        if let Some(mut entity_spatial) = self.world.get_mut::<SpatialComponent>(entity) {
            entity_spatial.velocity = velocity.clone();
            entity_spatial.attitude = attitude.clone();
        }
    }

    /// Set control inputs for the virtual aircraft
    pub fn set_controls(&mut self, entity: Entity, controls: &AircraftControlSurfaces) {
        if let Some(mut control_surfaces) = self.world.get_mut::<AircraftControlSurfaces>(entity) {
            *control_surfaces = controls.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{
        AircraftAeroCoefficients, AircraftGeometry, AircraftType, DragCoefficients,
        LiftCoefficients, MassModel, PitchCoefficients, PropulsionConfig,
    };
    use nalgebra::Matrix3;
    use std::f64::consts::PI;

    fn create_test_aircraft_config() -> FullAircraftConfig {
        let mass = 1000.0; // 1000 kg
        FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: AircraftType::Custom("TestAircraft".to_string()),
            mass: MassModel {
                mass,
                inertia: Matrix3::from_diagonal(&Vector3::new(1000.0, 2000.0, 1500.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 1000.0,
                    1.0 / 2000.0,
                    1.0 / 1500.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area: 16.0,
                wing_span: 10.0,
                mac: 1.6,
            },
            aero_coef: AircraftAeroCoefficients {
                lift: LiftCoefficients {
                    c_l_0: 0.2,
                    c_l_alpha: 5.0,
                    ..Default::default()
                },
                drag: DragCoefficients {
                    c_d_0: 0.02,
                    c_d_alpha2: 0.1,
                    ..Default::default()
                },
                pitch: PitchCoefficients {
                    c_m_0: 0.0,
                    c_m_alpha: -1.0,
                    c_m_q: -10.0,
                    c_m_deltae: -1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            propulsion: PropulsionConfig::twin_otter(),
            start_config: Default::default(),
            task_config: Default::default(),
        }
    }

    #[test]
    fn test_physics_initialization() {
        let physics_config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        };

        let mut virtual_physics = VirtualPhysics::new(&physics_config);

        // Create initial state
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0),
            velocity: Vector3::new(100.0, 0.0, 0.0),
            attitude: UnitQuaternion::from_euler_angles(0.0, 0.05, 0.0),
            angular_velocity: Vector3::zeros(),
        };

        let propulsion = PropulsionState::default();
        let config = create_test_aircraft_config();

        // Spawn aircraft and verify entity creation
        let entity = virtual_physics.spawn_aircraft(&spatial, &propulsion, &config);

        // Verify component setup
        let spawned_spatial = virtual_physics
            .world
            .get::<SpatialComponent>(entity)
            .expect("Entity should have SpatialComponent");
        let spawned_physics = virtual_physics
            .world
            .get::<PhysicsComponent>(entity)
            .expect("Entity should have PhysicsComponent");

        // Check initial state matches
        assert_eq!(spawned_spatial.velocity, spatial.velocity);
        assert_eq!(spawned_spatial.position, spatial.position);
        assert_eq!(spawned_physics.mass, config.mass.mass);
    }

    #[test]
    fn test_physics_step() {
        let physics_config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        };

        let mut virtual_physics = VirtualPhysics::new(&physics_config);

        // Create initial state with level flight
        let initial_velocity = 100.0;
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0),
            velocity: Vector3::new(initial_velocity, 0.0, 0.0),
            attitude: UnitQuaternion::from_euler_angles(0.0, 0.05, 0.0),
            angular_velocity: Vector3::zeros(),
        };

        let propulsion = PropulsionState::default();
        let config = create_test_aircraft_config();
        let entity = virtual_physics.spawn_aircraft(&spatial, &propulsion, &config);

        // Set trim-like control inputs
        virtual_physics.set_controls(
            entity,
            &AircraftControlSurfaces {
                elevator: -0.05,
                aileron: 0.0,
                rudder: 0.0,
                power_lever: 0.6,
            },
        );

        // Run single step
        virtual_physics.run_steps(entity, 1);

        // Get new state
        let (new_spatial, _) = virtual_physics.get_state(entity);

        // Verify reasonable physics behavior
        assert!(
            new_spatial.velocity.norm() > 0.0,
            "Velocity should not be zero"
        );
        assert!(
            new_spatial.velocity.norm() < physics_config.max_velocity,
            "Velocity should not exceed max velocity"
        );

        // Check conservation of energy (approximately)
        let initial_energy = 0.5 * config.mass.mass * initial_velocity.powi(2)
            + config.mass.mass * physics_config.gravity.norm() * 1000.0;
        let final_energy = 0.5 * config.mass.mass * new_spatial.velocity.norm().powi(2)
            + config.mass.mass * physics_config.gravity.norm() * (-new_spatial.position.z);

        assert!(
            (final_energy - initial_energy).abs() / initial_energy < 0.1,
            "Energy should be approximately conserved"
        );
    }

    #[test]
    fn test_force_calculation() {
        let physics_config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        };

        let mut virtual_physics = VirtualPhysics::new(&physics_config);

        // Test different flight conditions
        let test_conditions = vec![
            // Level flight
            (
                Vector3::new(100.0, 0.0, 0.0),
                UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            ),
            // Pitched up
            (
                Vector3::new(100.0, 0.0, 0.0),
                UnitQuaternion::from_euler_angles(0.0, PI / 8.0, 0.0),
            ),
            // Banked turn
            (
                Vector3::new(100.0, 0.0, 0.0),
                UnitQuaternion::from_euler_angles(PI / 6.0, 0.0, 0.0),
            ),
        ];

        let config = create_test_aircraft_config();
        let propulsion = PropulsionState::default();
        let base_spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        let entity = virtual_physics.spawn_aircraft(&base_spatial, &propulsion, &config);

        for (velocity, attitude) in test_conditions {
            // Set state
            virtual_physics.set_state(entity, &velocity, &attitude);

            // Calculate forces
            let (forces, moments) = virtual_physics.calculate_forces(entity);

            // Verify forces are finite
            assert!(
                forces.iter().all(|f| f.is_finite()),
                "Forces should be finite: {:?}",
                forces
            );
            assert!(
                moments.iter().all(|m| m.is_finite()),
                "Moments should be finite: {:?}",
                moments
            );

            // Verify force magnitudes are reasonable
            let dynamic_pressure = 0.5 * 1.225 * velocity.norm().powi(2);
            let max_expected_force = dynamic_pressure * config.geometry.wing_area * 3.0; // Reasonable max CL of 3.0

            assert!(
                forces.norm() < max_expected_force,
                "Force magnitude should be reasonable: {} vs {}",
                forces.norm(),
                max_expected_force
            );
        }
    }

    #[test]
    fn test_physics_reset() {
        // Test that the reset_forces function properly clears all forces

        let physics_config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        };

        let mut virtual_physics = VirtualPhysics::new(&physics_config);

        // Create initial state
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0),
            velocity: Vector3::new(100.0, 0.0, 0.0),
            attitude: UnitQuaternion::from_euler_angles(0.0, 0.05, 0.0),
            angular_velocity: Vector3::zeros(),
        };

        let propulsion = PropulsionState::default();
        let config = create_test_aircraft_config();
        let entity = virtual_physics.spawn_aircraft(&spatial, &propulsion, &config);

        // Calculate forces (this should populate the forces)
        virtual_physics.calculate_forces(entity);

        // Get physics component to check forces
        let physics_before = virtual_physics
            .world
            .get::<PhysicsComponent>(entity)
            .expect("Entity should have PhysicsComponent");

        // Verify forces are non-zero
        assert!(
            physics_before.net_force.norm() > 0.0,
            "Forces should be non-zero after calculation"
        );

        // Reset forces
        virtual_physics.reset_forces(entity);

        // Get physics component again
        let physics_after = virtual_physics
            .world
            .get::<PhysicsComponent>(entity)
            .expect("Entity should have PhysicsComponent");

        // Verify forces are now zero
        assert_eq!(
            physics_after.net_force,
            Vector3::zeros(),
            "Forces should be zero after reset"
        );

        assert_eq!(
            physics_after.net_moment,
            Vector3::zeros(),
            "Moments should be zero after reset"
        );

        assert_eq!(
            physics_after.forces.len(),
            0,
            "Force vector should be empty after reset"
        );
    }

    #[test]
    fn test_lateral_trim_forces() {
        // Test that the virtual physics correctly models lateral forces for banked flight

        let physics_config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        };

        let mut virtual_physics = VirtualPhysics::new(&physics_config);

        // Create initial state with bank angle
        let bank_angle = std::f64::consts::PI / 6.0; // 30 degrees
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0),
            velocity: Vector3::new(100.0, 0.0, 0.0),
            attitude: UnitQuaternion::from_euler_angles(bank_angle, 0.05, 0.0),
            angular_velocity: Vector3::zeros(),
        };

        let propulsion = PropulsionState::default();
        let config = create_test_aircraft_config();
        let entity = virtual_physics.spawn_aircraft(&spatial, &propulsion, &config);

        // Set control inputs
        virtual_physics.set_controls(
            entity,
            &AircraftControlSurfaces {
                elevator: -0.05,
                aileron: 0.1, // Add some aileron for roll stability
                rudder: 0.05, // Add some rudder for yaw stability
                power_lever: 0.6,
            },
        );

        // Calculate forces
        let (forces, moments) = virtual_physics.calculate_forces(entity);

        println!("Forces: {:?}", forces);
        println!("Moments: {:?}", moments);
        
        // In a banked turn, we should see non-zero lateral forces
        assert!(
            forces.y.abs() > 500.0, // Based on observed value of ~890
            "Banked attitude should produce significant lateral forces, got: {:.2}",
            forces.y
        );

        // We should also see roll and yaw moments
        assert!(
            moments.x.abs() > 200.0, // Based on observed value of ~400
            "Banked attitude should produce roll moments, got: {:.2}",
            moments.x
        );

        assert!(
            moments.z.abs() > 1000.0, // Based on observed value of ~8000
            "Banked attitude should produce yaw moments, got: {:.2}",
            moments.z
        );
    }
}
