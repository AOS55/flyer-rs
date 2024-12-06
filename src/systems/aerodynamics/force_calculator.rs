use aerso::{AeroEffect, AirState};
use bevy::prelude::*;
use nalgebra::Vector3;

use super::aerso_adapter::AersoAdapter;
use crate::components::{
    AerodynamicsComponent, Force, ForceCategory, Moment, PhysicsComponent, ReferenceFrame,
    SpatialComponent,
};
use crate::config::aerodynamics::AerodynamicsConfig;

/// System for calculating aerodynamic forces and moments
pub fn aero_force_system(
    mut query: Query<(
        &AerodynamicsComponent,
        &SpatialComponent,
        &mut PhysicsComponent,
    )>,
    config: Res<AerodynamicsConfig>,
) {
    for (aero, spatial, mut physics) in query.iter_mut() {
        if aero.air_data.true_airspeed < config.min_airspeed_threshold {
            continue;
        }

        let adapter = AersoAdapter::new(aero.geometry.clone(), aero.coefficients.clone());
        calculate_aero_forces(&adapter, aero, spatial, &mut physics);
    }
}

fn calculate_aero_forces(
    adapter: &AersoAdapter,
    aero: &AerodynamicsComponent,
    spatial: &SpatialComponent,
    physics: &mut PhysicsComponent,
) {
    let air_state = AirState {
        alpha: aero.air_data.alpha,
        beta: aero.air_data.beta,
        airspeed: aero.air_data.true_airspeed,
        q: aero.air_data.dynamic_pressure,
    };

    let input = vec![
        aero.control_surfaces.aileron,
        aero.control_surfaces.elevator,
        aero.control_surfaces.rudder,
        aero.control_surfaces.flaps,
    ];

    let (aero_force, aero_torque) = adapter.get_effect(air_state, spatial.angular_velocity, &input);

    let force_vector = match aero_force.frame {
        aerso::types::Frame::Body => aero_force.force,
        aerso::types::Frame::World => spatial.attitude.inverse() * aero_force.force,
    };

    physics.add_force(Force {
        vector: force_vector,
        point: None,
        frame: ReferenceFrame::Body,
        category: ForceCategory::Aerodynamic,
    });

    physics.add_moment(Moment {
        vector: aero_torque.torque,
        frame: ReferenceFrame::Body,
        category: ForceCategory::Aerodynamic,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{AircraftGeometry, ControlSurfaces};
    use crate::config::aerodynamics::AerodynamicsConfig;
    use approx::assert_relative_eq;
    use nalgebra::{Matrix3, UnitQuaternion};
    use std::f64::consts::PI;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<AerodynamicsConfig>();
        app
    }

    fn spawn_test_aircraft(app: &mut App) -> Entity {
        // Create default test components
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, 1000.0),
            velocity: Vector3::new(50.0, 0.0, 0.0), // 50 m/s forward flight
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        let mut aero = AerodynamicsComponent::default();
        aero.geometry = AircraftGeometry {
            wing_area: 16.0,
            wing_span: 10.0,
            mean_aerodynamic_chord: 1.6,
        };

        // Set realistic test coefficients
        aero.coefficients.lift.c_l_alpha = 5.0;
        aero.coefficients.drag.c_d_0 = 0.025;
        aero.coefficients.pitch.c_m_deltae = -1.5;

        // Initialize air data
        aero.air_data.true_airspeed = 50.0;
        aero.air_data.density = 1.225;
        aero.air_data.dynamic_pressure = 0.5 * 1.225 * 50.0 * 50.0;

        let physics = PhysicsComponent::new(
            1000.0,                       // 1000 kg mass
            Matrix3::identity() * 1000.0, // Simple inertia tensor
        );

        app.world.spawn((spatial, aero, physics)).id()
    }

    #[test]
    fn test_basic_force_calculation() {
        let mut app = setup_test_app();
        let entity = spawn_test_aircraft(&mut app);

        // Get initial component state
        let aero = app.world.get::<AerodynamicsComponent>(entity).unwrap();
        let spatial = app.world.get::<SpatialComponent>(entity).unwrap();

        info!("Initial conditions:");
        info!("Airspeed: {}", aero.air_data.true_airspeed);
        info!("Dynamic pressure: {}", aero.air_data.dynamic_pressure);

        // Run the system
        app.add_systems(Update, aero_force_system);
        app.update();

        // Check results
        let physics = app.world.get::<PhysicsComponent>(entity).unwrap();

        info!("Resulting forces:");
        info!("Net force: {:?}", physics.net_force);
        info!("Net moment: {:?}", physics.net_moment);

        assert!(
            physics.forces.len() > 0,
            "Expected aerodynamic forces to be added"
        );
        assert!(
            physics.forces[0].vector.norm() > 0.0,
            "Expected non-zero force magnitude"
        );
    }

    #[test]
    fn test_zero_airspeed_condition() {
        let mut app = setup_test_app();

        // Spawn aircraft with zero airspeed
        let entity = app
            .world
            .spawn((
                SpatialComponent {
                    velocity: Vector3::zeros(),
                    ..Default::default()
                },
                AerodynamicsComponent {
                    air_data: crate::components::AirData {
                        true_airspeed: 0.0,
                        dynamic_pressure: 0.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                PhysicsComponent::new(1000.0, Matrix3::identity() * 1000.0),
            ))
            .id();

        // Run the system
        app.add_systems(Update, aero_force_system);
        app.update();

        // Check results
        let physics = app.world.get::<PhysicsComponent>(entity).unwrap();
        assert_eq!(
            physics.forces.len(),
            0,
            "Expected no forces at zero airspeed"
        );
    }

    #[test]
    fn test_control_surface_moments() {
        let mut app = setup_test_app();

        // Spawn aircraft with elevator deflection
        let mut aero = AerodynamicsComponent::default();
        aero.control_surfaces.elevator = 0.1; // 10% deflection
        aero.air_data.true_airspeed = 50.0;
        aero.air_data.dynamic_pressure = 0.5 * 1.225 * 50.0 * 50.0;

        let entity = app
            .world
            .spawn((
                SpatialComponent::default(),
                aero,
                PhysicsComponent::new(1000.0, Matrix3::identity() * 1000.0),
            ))
            .id();

        // Run the system
        app.add_systems(Update, aero_force_system);
        app.update();

        // Check results
        let physics = app.world.get::<PhysicsComponent>(entity).unwrap();
        assert!(
            physics.moments.len() > 0,
            "Expected moments from control surface deflection"
        );

        let pitch_moment = physics.moments[0].vector.y;
        assert!(
            pitch_moment.abs() > 0.0,
            "Expected non-zero pitch moment from elevator"
        );
    }

    #[test]
    fn test_attitude_effects() {
        let mut app = setup_test_app();

        // Spawn aircraft with pitched attitude
        let attitude = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 4.0); // 45° pitch
        let entity = app
            .world
            .spawn((
                SpatialComponent {
                    attitude,
                    velocity: Vector3::new(50.0, 0.0, 0.0),
                    ..Default::default()
                },
                AerodynamicsComponent {
                    air_data: crate::components::AirData {
                        true_airspeed: 50.0,
                        dynamic_pressure: 0.5 * 1.225 * 50.0 * 50.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                PhysicsComponent::new(1000.0, Matrix3::identity() * 1000.0),
            ))
            .id();

        // Run the system
        app.add_systems(Update, aero_force_system);
        app.update();

        // Check results
        let physics = app.world.get::<PhysicsComponent>(entity).unwrap();
        assert!(
            physics.forces.len() > 0,
            "Expected forces with pitched attitude"
        );

        // Forces should be in body frame
        let force = &physics.forces[0];
        assert_eq!(
            force.frame,
            ReferenceFrame::Body,
            "Forces should be in body frame"
        );
    }

    #[test]
    fn test_combined_effects() {
        let mut app = setup_test_app();

        // Create aircraft with complex flight condition
        let mut aero = AerodynamicsComponent::default();
        aero.air_data.true_airspeed = 50.0;
        aero.air_data.alpha = 5.0 * PI / 180.0;
        aero.air_data.beta = 2.0 * PI / 180.0;
        aero.air_data.dynamic_pressure = 0.5 * 1.225 * 50.0 * 50.0;
        aero.control_surfaces.elevator = 0.05;
        aero.control_surfaces.aileron = 0.03;
        aero.control_surfaces.rudder = 0.02;

        let entity = app
            .world
            .spawn((
                SpatialComponent {
                    velocity: Vector3::new(49.0, 2.0, 4.0),
                    angular_velocity: Vector3::new(0.1, 0.05, 0.02),
                    ..Default::default()
                },
                aero,
                PhysicsComponent::new(1000.0, Matrix3::identity() * 1000.0),
            ))
            .id();

        // Run the system
        app.add_systems(Update, aero_force_system);
        app.update();

        // Check results
        let physics = app.world.get::<PhysicsComponent>(entity).unwrap();

        assert!(physics.forces.len() > 0, "Expected aerodynamic forces");
        assert!(physics.moments.len() > 0, "Expected aerodynamic moments");

        // Verify we have forces and moments in all axes
        let force = &physics.forces[0].vector;
        let moment = &physics.moments[0].vector;

        assert!(force.x.abs() > 0.0, "Expected force in x-axis");
        assert!(force.y.abs() > 0.0, "Expected force in y-axis");
        assert!(force.z.abs() > 0.0, "Expected force in z-axis");

        assert!(moment.x.abs() > 0.0, "Expected roll moment");
        assert!(moment.y.abs() > 0.0, "Expected pitch moment");
        assert!(moment.z.abs() > 0.0, "Expected yaw moment");
    }
}
