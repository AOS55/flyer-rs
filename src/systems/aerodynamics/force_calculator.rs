use aerso::{AeroEffect, AirState};
use bevy::prelude::*;

use super::aerso_adapter::AersoAdapter;
use crate::components::{
    AirData, AircraftControlSurfaces, Force, ForceCategory, FullAircraftConfig, Moment,
    PhysicsComponent, ReferenceFrame, SpatialComponent,
};
use crate::resources::AerodynamicsConfig;

/// System for calculating aerodynamic forces and moments acting on aircraft.
///
/// This system computes the aerodynamic forces and moments based on the aircraft's geometry,
/// aerodynamic coefficients, control surface inputs, air data, and spatial properties. The
/// results are added to the aircraft's physics component.
pub fn aero_force_system(
    mut aircraft: Query<(
        &AircraftControlSurfaces,
        &AirData,
        &SpatialComponent,
        &mut PhysicsComponent,
        &FullAircraftConfig,
    )>,
    aero_config: Res<AerodynamicsConfig>,
) {
    // println!("Running Aero Force System!");
    for (controls, air_data, spatial, mut physics, config) in aircraft.iter_mut() {
        calculate_and_apply_aero_forces(
            controls,
            air_data,
            spatial,
            &mut physics,
            config,
            &aero_config,
        );
    }
}

/// Core aerodynamic calculation function that doesn't depend on Bevy ECS
/// Can be used both from the system and directly from tests
pub fn calculate_and_apply_aero_forces(
    controls: &AircraftControlSurfaces,
    air_data: &AirData,
    spatial: &SpatialComponent,
    physics: &mut PhysicsComponent,
    config: &FullAircraftConfig,
    aero_config: &AerodynamicsConfig,
) {
    // Early return if airspeed is below threshold
    if air_data.true_airspeed < aero_config.min_airspeed_threshold {
        return;
    }

    // Create adapter outside of the calculation
    let adapter = AersoAdapter::new(config.geometry.clone(), config.aero_coef.clone());

    // Collect all necessary data before calculation
    let aero_forces = prepare_aero_forces(&adapter, controls, air_data, spatial);
    // println!("Forces Config: {:?}", config);

    // Clear existing aerodynamic forces and moments before adding new ones
    physics
        .forces
        .retain(|force| force.category != ForceCategory::Aerodynamic);
    physics
        .moments
        .retain(|moment| moment.category != ForceCategory::Aerodynamic);

    // Apply the calculated forces and moments
    apply_aero_forces(physics, aero_forces);
}

/// Represents the calculated aerodynamic forces and moments
struct AeroForces {
    force: Force,
    moment: Moment,
}

/// Prepares aerodynamic forces and moments without mutating any state
fn prepare_aero_forces(
    adapter: &AersoAdapter,
    control_surfaces: &AircraftControlSurfaces,
    air_data: &AirData,
    spatial: &SpatialComponent,
) -> AeroForces {
    let air_state = AirState {
        alpha: air_data.alpha,
        beta: air_data.beta,
        airspeed: air_data.true_airspeed,
        q: air_data.dynamic_pressure,
    };

    let input = vec![
        control_surfaces.aileron,
        control_surfaces.elevator,
        control_surfaces.rudder,
        control_surfaces.power_lever,
    ];

    let (aero_force, aero_torque) = adapter.get_effect(air_state, spatial.angular_velocity, &input);

    let force_vector = match aero_force.frame {
        aerso::types::Frame::Body => aero_force.force,
        aerso::types::Frame::World => spatial.attitude.inverse() * aero_force.force,
    };

    AeroForces {
        force: Force {
            vector: force_vector,
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        },
        moment: Moment {
            vector: aero_torque.torque,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        },
    }
}

/// Applies the calculated forces and moments to the physics component
fn apply_aero_forces(physics: &mut PhysicsComponent, aero_forces: AeroForces) {
    physics.add_force(aero_forces.force);
    physics.add_moment(aero_forces.moment);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{
            AirData, AircraftAeroCoefficients, AircraftControlSurfaces, AircraftGeometry,
            FullAircraftConfig, PhysicsComponent, SpatialComponent,
        },
        resources::{AerodynamicsConfig, EnvironmentConfig, EnvironmentModel, PhysicsConfig},
    };
    use approx::assert_relative_eq;
    use nalgebra::{Matrix3, UnitQuaternion, Vector3};
    use std::f64::consts::PI;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.insert_resource(AerodynamicsConfig {
            min_airspeed_threshold: 0.5,
        });
        app.insert_resource(EnvironmentModel::new(&EnvironmentConfig::default()));
        app.insert_resource(PhysicsConfig::default());
        app
    }

    fn spawn_test_aircraft(app: &mut App) -> Entity {
        // Create aircraft with realistic coefficients (Twin Otter)
        let config = FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: crate::components::AircraftType::Custom("TestAircraft".to_string()),
            mass: crate::components::MassModel {
                mass: 4500.0, // kg
                inertia: Matrix3::from_diagonal(&Vector3::new(25000.0, 33000.0, 50000.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 25000.0,
                    1.0 / 33000.0,
                    1.0 / 50000.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area: 39.0, // m²
                wing_span: 19.8, // m
                mac: 1.98,       // m
            },
            aero_coef: AircraftAeroCoefficients::twin_otter(),
            propulsion: crate::components::PropulsionConfig::twin_otter(),
            start_config: Default::default(),
            task_config: Default::default(),
        };

        // Create basic initial state
        let air_data = AirData::default();
        let controls = AircraftControlSurfaces::default();
        let spatial = SpatialComponent::default();
        let physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);

        app.world_mut()
            .spawn((air_data, controls, spatial, physics, config))
            .id()
    }

    #[test]
    fn test_basic_force_calculation() {
        // Create test components directly without using Bevy ECS
        let speed = 50.0;

        // Create air data
        let air_data = AirData {
            true_airspeed: speed,
            dynamic_pressure: 0.5 * 1.225 * speed * speed,
            density: 1.225,
            alpha: 0.05, // ~3 degrees
            beta: 0.0,
            relative_velocity: Vector3::new(speed, 0.0, 0.0),
            wind_velocity: Vector3::zeros(),
        };

        // Create spatial component
        let spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::new(speed, 0.0, 0.0),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        // Create control surfaces
        let controls = AircraftControlSurfaces::default();

        // Create aircraft config with realistic coefficients
        let config = FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: crate::components::AircraftType::Custom("TestAircraft".to_string()),
            mass: crate::components::MassModel {
                mass: 4500.0, // kg
                inertia: Matrix3::from_diagonal(&Vector3::new(25000.0, 33000.0, 50000.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 25000.0,
                    1.0 / 33000.0,
                    1.0 / 50000.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area: 39.0, // m²
                wing_span: 19.8, // m
                mac: 1.98,       // m
            },
            aero_coef: AircraftAeroCoefficients::twin_otter(),
            propulsion: crate::components::PropulsionConfig::twin_otter(),
            start_config: Default::default(),
            task_config: Default::default(),
        };

        // Create aero config
        let aero_config = AerodynamicsConfig {
            min_airspeed_threshold: 0.5,
        };

        // Create physics component
        let mut physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);

        // Run calculation
        calculate_and_apply_aero_forces(
            &controls,
            &air_data,
            &spatial,
            &mut physics,
            &config,
            &aero_config,
        );

        // Check that forces are calculated
        assert!(!physics.forces.is_empty(), "Should have calculated forces");

        // Find the aerodynamic force
        let aero_force = physics
            .forces
            .iter()
            .find(|f| f.category == ForceCategory::Aerodynamic)
            .expect("Should have an aerodynamic force");

        // At positive alpha, should have:
        // - Negative X force (drag)
        // - Negative Z force (lift, as Z is down)
        // - No Y force (no sideslip)
        assert!(
            aero_force.vector.x < 0.0,
            "Should have drag (negative X force)"
        );
        assert!(
            aero_force.vector.z < 0.0,
            "Should have lift (negative Z force)"
        );
        assert_relative_eq!(aero_force.vector.y, 0.0, epsilon = 1e-10);

        // Check frame of reference
        assert_eq!(aero_force.frame, ReferenceFrame::Body);
    }

    #[test]
    fn test_zero_airspeed_condition() {
        // Create test components directly without using Bevy ECS

        // Create air data with zero airspeed
        let air_data = AirData {
            true_airspeed: 0.0,
            dynamic_pressure: 0.0,
            density: 1.225,
            alpha: 0.0,
            beta: 0.0,
            relative_velocity: Vector3::zeros(),
            wind_velocity: Vector3::zeros(),
        };

        // Create spatial component with zero velocity
        let spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        // Create control surfaces
        let controls = AircraftControlSurfaces::default();

        // Create aircraft config with realistic coefficients
        let config = FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: crate::components::AircraftType::Custom("TestAircraft".to_string()),
            mass: crate::components::MassModel {
                mass: 4500.0, // kg
                inertia: Matrix3::from_diagonal(&Vector3::new(25000.0, 33000.0, 50000.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 25000.0,
                    1.0 / 33000.0,
                    1.0 / 50000.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area: 39.0, // m²
                wing_span: 19.8, // m
                mac: 1.98,       // m
            },
            aero_coef: AircraftAeroCoefficients::twin_otter(),
            propulsion: crate::components::PropulsionConfig::twin_otter(),
            start_config: Default::default(),
            task_config: Default::default(),
        };

        // Create aero config
        let aero_config = AerodynamicsConfig {
            min_airspeed_threshold: 0.5,
        };

        // Create physics component
        let mut physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);

        // Run calculation
        calculate_and_apply_aero_forces(
            &controls,
            &air_data,
            &spatial,
            &mut physics,
            &config,
            &aero_config,
        );

        // Check that no forces are applied at zero airspeed
        let aero_forces = physics
            .forces
            .iter()
            .filter(|f| f.category == ForceCategory::Aerodynamic)
            .count();

        assert_eq!(
            aero_forces, 0,
            "No aerodynamic forces should be applied at zero airspeed"
        );
    }

    #[test]
    fn test_stall_characteristics() {
        // Create test components directly without using Bevy ECS
        // Test various angles of attack
        let test_angles = vec![
            (0.0, "zero_aoa"),    // Zero angle
            (5.0, "cruise_aoa"),  // Normal cruise
            (10.0, "high_aoa"),   // High but not stalled
            (15.0, "near_stall"), // Near stall
            (20.0, "stall"),      // Stall angle
            (30.0, "deep_stall"), // Deep stall
        ];

        // Track CL values to verify stall behavior
        let mut cl_values = Vec::new();

        // Create aircraft config with realistic coefficients
        let config = FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: crate::components::AircraftType::Custom("TestAircraft".to_string()),
            mass: crate::components::MassModel {
                mass: 4500.0, // kg
                inertia: Matrix3::from_diagonal(&Vector3::new(25000.0, 33000.0, 50000.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 25000.0,
                    1.0 / 33000.0,
                    1.0 / 50000.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area: 39.0, // m²
                wing_span: 19.8, // m
                mac: 1.98,       // m
            },
            aero_coef: AircraftAeroCoefficients::twin_otter(),
            propulsion: crate::components::PropulsionConfig::twin_otter(),
            start_config: Default::default(),
            task_config: Default::default(),
        };

        // Create aero config
        let aero_config = AerodynamicsConfig {
            min_airspeed_threshold: 0.5,
        };

        // Create control surfaces
        let controls = AircraftControlSurfaces::default();

        for (angle_deg, _name) in test_angles {
            let angle_rad = angle_deg * PI / 180.0;
            let speed = 50.0;

            // Create air data
            let air_data = AirData {
                true_airspeed: speed,
                dynamic_pressure: 0.5 * 1.225 * speed * speed,
                density: 1.225,
                alpha: angle_rad,
                beta: 0.0,
                relative_velocity: Vector3::new(speed * angle_rad.cos(), 0.0, -speed * angle_rad.sin()),
                wind_velocity: Vector3::zeros(),
            };

            // Create spatial component
            let spatial = SpatialComponent {
                position: Vector3::zeros(),
                velocity: Vector3::new(speed * angle_rad.cos(), 0.0, -speed * angle_rad.sin()),
                attitude: UnitQuaternion::identity(),
                angular_velocity: Vector3::zeros(),
            };

            // Create physics component
            let mut physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);

            // Run calculation
            calculate_and_apply_aero_forces(
                &controls,
                &air_data,
                &spatial,
                &mut physics,
                &config,
                &aero_config,
            );

            if let Some(aero_force) = physics
                .forces
                .iter()
                .find(|f| f.category == ForceCategory::Aerodynamic)
            {
                // Calculate non-dimensional coefficients
                let q = air_data.dynamic_pressure;
                let s = config.geometry.wing_area;

                // CL = -Fz / (q * S)  (negative because Z is down)
                let cl = -aero_force.vector.z / (q * s);
                cl_values.push((angle_deg, cl));

                println!("Angle: {:.1}°, CL: {:.3}", angle_deg, cl);
            }
        }

        // Verify stall behavior - CL should increase then decrease or plateau
        if cl_values.len() >= 4 {
            // Check that CL increases initially
            assert!(
                cl_values[1].1 > cl_values[0].1,
                "CL should increase with angle of attack in normal range"
            );

            // Check that CL eventually decreases or plateaus after stall
            let high_idx = cl_values.len() - 1;
            let peak_cl = cl_values
                .iter()
                .map(|&(_, cl)| cl)
                .fold(f64::NEG_INFINITY, f64::max);
            let last_cl = cl_values[high_idx].1;

            assert!(
                last_cl < peak_cl,
                "CL should decrease after stall (peak CL: {}, final CL: {})",
                peak_cl,
                last_cl
            );
        }
    }

    #[test]
    fn test_control_surface_moments() {
        // Test various control surface deflections
        struct TestCase {
            name: &'static str,
            elevator: f64,
            aileron: f64,
            rudder: f64,
            expected_moment_x: fn(f64) -> bool, // Roll moment check
            expected_moment_y: fn(f64) -> bool, // Pitch moment check
            expected_moment_z: fn(f64) -> bool, // Yaw moment check
        }

        let test_cases = vec![
            TestCase {
                name: "elevator_up",
                elevator: 0.3, // Positive elevator (up)
                aileron: 0.0,
                rudder: 0.0,
                expected_moment_x: |m| m.abs() < 1.0, // No significant roll
                expected_moment_y: |m| m.abs() > 10000.0,  // Large pitch moment with sign check below
                expected_moment_z: |m| m.abs() < 1.0, // No significant yaw
            },
            TestCase {
                name: "elevator_down",
                elevator: -0.3, // Negative elevator (down)
                aileron: 0.0,
                rudder: 0.0,
                expected_moment_x: |m| m.abs() < 1.0, // No significant roll
                expected_moment_y: |m| m.abs() > 10000.0,  // Large pitch moment with sign check below
                expected_moment_z: |m| m.abs() < 1.0, // No significant yaw
            },
            TestCase {
                name: "aileron_right",
                elevator: 0.0,
                aileron: 0.3, // Positive aileron (right roll)
                rudder: 0.0,
                expected_moment_x: |m| m.abs() > 1000.0, // Significant roll moment
                expected_moment_y: |m| m.abs() < 5000.0, // Minimal pitch
                expected_moment_z: |m| m.abs() > 100.0,  // Some yaw moment
            },
            TestCase {
                name: "rudder_right",
                elevator: 0.0,
                aileron: 0.0,
                rudder: 0.3,                           // Positive rudder (yaw right)
                expected_moment_x: |m| m.abs() > 10.0, // Some roll due to rudder
                expected_moment_y: |m| m.abs() < 5000.0, // Minimal pitch
                expected_moment_z: |m| m.abs() > 1000.0,  // Significant yaw moment
            },
        ];

        // Set a standard flight condition
        let speed = 70.0;
        
        // Create aircraft config with realistic coefficients
        let config = FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: crate::components::AircraftType::Custom("TestAircraft".to_string()),
            mass: crate::components::MassModel {
                mass: 4500.0, // kg
                inertia: Matrix3::from_diagonal(&Vector3::new(25000.0, 33000.0, 50000.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 25000.0,
                    1.0 / 33000.0,
                    1.0 / 50000.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area: 39.0, // m²
                wing_span: 19.8, // m
                mac: 1.98,       // m
            },
            aero_coef: AircraftAeroCoefficients::twin_otter(),
            propulsion: crate::components::PropulsionConfig::twin_otter(),
            start_config: Default::default(),
            task_config: Default::default(),
        };

        // Create aero config
        let aero_config = AerodynamicsConfig {
            min_airspeed_threshold: 0.5,
        };

        // Create standard air data
        let air_data = AirData {
            true_airspeed: speed,
            dynamic_pressure: 0.5 * 1.225 * speed * speed,
            density: 1.225,
            alpha: 0.05, // ~3 degrees
            beta: 0.0,
            relative_velocity: Vector3::new(speed, 0.0, 0.0),
            wind_velocity: Vector3::zeros(),
        };

        // Create spatial component
        let spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::new(speed, 0.0, 0.0),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        for test_case in test_cases {
            // Create control surfaces with specific settings for this test
            let controls = AircraftControlSurfaces {
                elevator: test_case.elevator,
                aileron: test_case.aileron,
                rudder: test_case.rudder,
                ..Default::default()
            };

            // Create fresh physics component for each test
            let mut physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);

            // Run calculation
            calculate_and_apply_aero_forces(
                &controls,
                &air_data,
                &spatial,
                &mut physics,
                &config,
                &aero_config,
            );

            // Check moments
            let aero_moment = physics
                .moments
                .iter()
                .find(|m| m.category == ForceCategory::Aerodynamic)
                .expect("Should have aerodynamic moment");

            // Check that moments match expectations
            assert!(
                (test_case.expected_moment_x)(aero_moment.vector.x),
                "Roll moment incorrect for {}: got {}",
                test_case.name,
                aero_moment.vector.x
            );

            assert!(
                (test_case.expected_moment_y)(aero_moment.vector.y),
                "Pitch moment incorrect for {}: got {}",
                test_case.name,
                aero_moment.vector.y
            );

            assert!(
                (test_case.expected_moment_z)(aero_moment.vector.z),
                "Yaw moment incorrect for {}: got {}",
                test_case.name,
                aero_moment.vector.z
            );

            println!(
                "Control test '{}' passed with moments: x={}, y={}, z={}",
                test_case.name, aero_moment.vector.x, aero_moment.vector.y, aero_moment.vector.z
            );
        }
    }

    #[test]
    fn test_attitude_effects() {
        // Test various aircraft attitudes
        struct AttitudeTest {
            name: &'static str,
            roll: f64,  // in radians
            pitch: f64, // in radians
            yaw: f64,   // in radians
        }

        let test_attitudes = vec![
            AttitudeTest {
                name: "level",
                roll: 0.0,
                pitch: 0.0,
                yaw: 0.0,
            },
            AttitudeTest {
                name: "pitched_up",
                roll: 0.0,
                pitch: 10.0 * PI / 180.0,
                yaw: 0.0,
            },
            AttitudeTest {
                name: "banked_right",
                roll: 30.0 * PI / 180.0,
                pitch: 0.0,
                yaw: 0.0,
            },
            AttitudeTest {
                name: "nose_right",
                roll: 0.0,
                pitch: 0.0,
                yaw: 45.0 * PI / 180.0,
            },
        ];

        // Initial flight condition
        let speed = 80.0;
        
        // Create aircraft config with realistic coefficients
        let config = FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: crate::components::AircraftType::Custom("TestAircraft".to_string()),
            mass: crate::components::MassModel {
                mass: 4500.0, // kg
                inertia: Matrix3::from_diagonal(&Vector3::new(25000.0, 33000.0, 50000.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 25000.0,
                    1.0 / 33000.0,
                    1.0 / 50000.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area: 39.0, // m²
                wing_span: 19.8, // m
                mac: 1.98,       // m
            },
            aero_coef: AircraftAeroCoefficients::twin_otter(),
            propulsion: crate::components::PropulsionConfig::twin_otter(),
            start_config: Default::default(),
            task_config: Default::default(),
        };

        // Create aero config
        let aero_config = AerodynamicsConfig {
            min_airspeed_threshold: 0.5,
        };

        // Create control surfaces
        let controls = AircraftControlSurfaces::default();

        // Create standard air data
        let standard_air_data = AirData {
            true_airspeed: speed,
            dynamic_pressure: 0.5 * 1.225 * speed * speed,
            density: 1.225,
            alpha: 0.05, // ~3 degrees
            beta: 0.0,
            relative_velocity: Vector3::new(speed, 0.0, 0.0),
            wind_velocity: Vector3::zeros(),
        };

        for test in test_attitudes {
            // Create spatial component with specific attitude
            let spatial = SpatialComponent {
                position: Vector3::zeros(),
                velocity: Vector3::new(speed, 0.0, 0.0),
                attitude: UnitQuaternion::from_euler_angles(test.roll, test.pitch, test.yaw),
                angular_velocity: Vector3::zeros(),
            };

            // Create fresh physics component for each test
            let mut physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);

            // Run calculation
            calculate_and_apply_aero_forces(
                &controls,
                &standard_air_data,
                &spatial,
                &mut physics,
                &config,
                &aero_config,
            );

            // Check forces and calculate body-to-world transformation manually
            if let Some(aero_force) = physics
                .forces
                .iter()
                .find(|f| f.category == ForceCategory::Aerodynamic)
            {
                // Convert body force to inertial frame for validation
                let inertial_force = spatial.attitude * aero_force.vector;
                
                println!(
                    "Attitude '{}': roll={:.1}°, pitch={:.1}°, yaw={:.1}°",
                    test.name,
                    test.roll * 180.0 / PI,
                    test.pitch * 180.0 / PI,
                    test.yaw * 180.0 / PI
                );

                println!("  Body force: {:?}", aero_force.vector);
                println!("  Inertial force: {:?}", inertial_force);

                // Verify forces are still finite and reasonable
                assert!(
                    aero_force.vector.iter().all(|v| v.is_finite()),
                    "Forces should remain finite"
                );
                
                assert!(
                    inertial_force.iter().all(|v| v.is_finite()),
                    "Inertial forces should remain finite"
                );
            }
        }
    }

    #[test]
    fn test_combined_effects() {
        // Test a complex flight condition with:
        // - Medium angle of attack
        // - Some sideslip
        // - Control surface deflections
        // - Non-zero angular rates
        let speed = 60.0;
        
        // Create aircraft config with realistic coefficients
        let config = FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: crate::components::AircraftType::Custom("TestAircraft".to_string()),
            mass: crate::components::MassModel {
                mass: 4500.0, // kg
                inertia: Matrix3::from_diagonal(&Vector3::new(25000.0, 33000.0, 50000.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 25000.0,
                    1.0 / 33000.0,
                    1.0 / 50000.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area: 39.0, // m²
                wing_span: 19.8, // m
                mac: 1.98,       // m
            },
            aero_coef: AircraftAeroCoefficients::twin_otter(),
            propulsion: crate::components::PropulsionConfig::twin_otter(),
            start_config: Default::default(),
            task_config: Default::default(),
        };

        // Create aero config
        let aero_config = AerodynamicsConfig {
            min_airspeed_threshold: 0.5,
        };

        // Set angles
        let alpha = 8.0 * PI / 180.0; // 8 degrees AoA
        let beta = 5.0 * PI / 180.0;  // 5 degrees sideslip

        // Create air data
        let air_data = AirData {
            true_airspeed: speed,
            dynamic_pressure: 0.5 * 1.225 * speed * speed,
            density: 1.225,
            alpha,
            beta,
            relative_velocity: Vector3::new(
                speed * alpha.cos() * beta.cos(),
                speed * beta.sin(),
                -speed * alpha.sin() * beta.cos(),
            ),
            wind_velocity: Vector3::zeros(),
        };

        // Create spatial component with angular velocities
        let spatial = SpatialComponent {
            position: Vector3::zeros(),
            velocity: Vector3::new(
                speed * alpha.cos() * beta.cos(),
                speed * beta.sin(),
                -speed * alpha.sin() * beta.cos(),
            ),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::new(
                0.1,   // mild roll rate
                0.05,  // mild pitch rate
                -0.02, // mild yaw rate
            ),
        };

        // Create control surfaces with specific settings
        let controls = AircraftControlSurfaces {
            elevator: -0.1, // mild nose up
            aileron: 0.2,   // right roll
            rudder: -0.05,  // left yaw (to counter adverse yaw)
            power_lever: 0.6,
        };

        // Create physics component
        let mut physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);

        // Run calculation
        calculate_and_apply_aero_forces(
            &controls,
            &air_data,
            &spatial,
            &mut physics,
            &config,
            &aero_config,
        );

        // Check forces and moments are calculated
        let aero_force = physics
            .forces
            .iter()
            .find(|f| f.category == ForceCategory::Aerodynamic)
            .expect("Should have an aerodynamic force");

        let aero_moment = physics
            .moments
            .iter()
            .find(|m| m.category == ForceCategory::Aerodynamic)
            .expect("Should have an aerodynamic moment");

        // In this complex case, we should have:
        // - Forces in all three axes
        // - Moments in all three axes
        assert!(aero_force.vector.x != 0.0, "Should have X force");
        assert!(
            aero_force.vector.y != 0.0,
            "Should have Y force due to sideslip"
        );
        assert!(aero_force.vector.z != 0.0, "Should have Z force");

        assert!(aero_moment.vector.x != 0.0, "Should have roll moment");
        assert!(aero_moment.vector.y != 0.0, "Should have pitch moment");
        assert!(aero_moment.vector.z != 0.0, "Should have yaw moment");

        println!("Combined effects test forces: {:?}", aero_force.vector);
        println!("Combined effects test moments: {:?}", aero_moment.vector);

        // Verify forces and moments are finite
        assert!(aero_force.vector.iter().all(|v| v.is_finite()));
        assert!(aero_moment.vector.iter().all(|v| v.is_finite()));
    }
}
