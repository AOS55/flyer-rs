use approx::assert_relative_eq;
use bevy::prelude::*;
use flyer::{
    components::{AirData, DubinsAircraftState, FullAircraftState, NeedsTrim, TrimCondition},
    resources::PhysicsConfig,
};
use nalgebra::Vector3;

use crate::common::{
    assert_full_state_valid, create_test_full_config, neutral_controls, wait_for_condition,
    TestAppBuilder,
};

#[test]
fn test_straight_level_flight() {
    let aircraft_config = create_test_full_config();
    let mut app = TestAppBuilder::new()
        .with_full_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    // Run for 1 second of simulation time
    app.run_steps(120);

    if let Some(state) = app.query_single::<FullAircraftState>() {
        assert_full_state_valid(state);

        // Verify altitude is maintained
        let initial_altitude = -500.0;
        assert_relative_eq!(state.spatial.position.z, initial_altitude, epsilon = 1.0);

        // Verify forward motion
        assert!(state.spatial.velocity.x > 0.0);
    } else {
        panic!("Aircraft state not found");
    }

    // // Add trim request for S+L flight at 50 m/s
    // let trimmed = wait_for_condition(
    //     &mut app,
    //     |app| {
    //         let world = app.world_mut();

    //         let entity = world
    //             .query_filtered::<Entity, With<FullAircraftState>>()
    //             .get_single(world)
    //             .ok();

    //         if let Some(entity) = entity {
    //             world.entity_mut(entity).insert(NeedsTrim {
    //                 condition: TrimCondition::StraightAndLevel { airspeed: 50.0 },
    //                 solver: None,
    //             });
    //             true
    //         } else {
    //             false
    //         }
    //     },
    //     10,
    // );

    // assert!(trimmed, "Failed to request trim");

    // // Run simulation until trim converges (max 1000 steps)
    // let trimmed = wait_for_condition(
    //     &mut app,
    //     |app| {
    //         app.world_mut()
    //             .query_filtered::<Entity, With<NeedsTrim>>()
    //             .iter(&app.world())
    //             .next()
    //             .is_none()
    //     },
    //     1000,
    // );

    // assert!(trimmed, "Trim did not converge");

    // // Get aircraft state and validate
    // if let Some(state) = app.query_single::<FullAircraftState>() {
    //     assert_full_state_valid(state);

    //     // Verify straight and level conditions
    //     let velocity = state.spatial.velocity;
    //     let airspeed = velocity.norm();
    //     println!("State: {:?}", state);
    //     println!("velocity: {}, airspeed: {}", velocity, airspeed);
    //     assert!((airspeed - 50.0).abs() < 1.0, "Airspeed not at target");

    //     // Check vertical speed is close to zero
    //     assert!(velocity.z.abs() < 0.1, "Significant vertical speed present");

    //     // Check roll and pitch angles are small
    //     let (roll, pitch, _) = state.spatial.attitude.euler_angles();
    //     assert!(roll.abs() < 0.01, "Significant roll angle present");
    //     assert!(pitch.abs() < 0.1, "Significant pitch angle present");
    // } else {
    //     panic!("Could not find aircraft state");
    // }

    // // Run for 10 seconds and verify stable flight
    // app.run_steps((10.0 / app.steps_per_action as f64) as usize);

    // if let Some(final_state) = app.query_single::<FullAircraftState>() {
    //     assert_full_state_valid(final_state);

    //     let velocity = final_state.spatial.velocity;
    //     let airspeed = velocity.norm();
    //     assert!((airspeed - 50.0).abs() < 1.0, "Failed to maintain airspeed");
    //     assert!(velocity.z.abs() < 0.1, "Failed to maintain altitude");
    // } else {
    //     panic!("Could not find final aircraft state");
    // }
}

#[test]
fn test_elevator_control() {
    let aircraft_config = create_test_full_config();

    // Print the aerodynamic coefficients we're using
    println!("Aircraft config:");
    println!(
        "  Pitch coefficients: {:?}",
        aircraft_config.aero_coef.pitch
    );
    println!("  Lift coefficients: {:?}", aircraft_config.aero_coef.lift);
    println!("  Geometry: {:?}", aircraft_config.geometry);

    let mut app = TestAppBuilder::new()
        .with_full_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    // Initialize with steady flight conditions
    if let Some(mut state) = app.query_single_mut::<FullAircraftState>() {
        // Set initial conditions for stable flight
        state.spatial.velocity = Vector3::new(50.0, 0.0, 0.0);
        state.air_data = AirData {
            true_airspeed: 50.0,
            alpha: 0.0,
            beta: 0.0,
            dynamic_pressure: 0.5 * 1.225 * 50.0 * 50.0,
            density: 1.225,
            relative_velocity: Vector3::new(50.0, 0.0, 0.0),
            wind_velocity: Vector3::zeros(),
        };

        // Print initial forces and moments
        println!("Initial forces: {:?}", state.physics.forces);
        println!("Initial moments: {:?}", state.physics.moments);

        // Set trim condition
        let mut controls = neutral_controls();
        controls.power_lever = 0.5; // Add power to maintain flight
        state.control_surfaces = controls;
    }

    // Run for a short time to stabilize
    app.run_steps(60);

    // Get initial pitch attitude
    let initial_pitch = if let Some(state) = app.query_single::<FullAircraftState>() {
        let (_roll, pitch, _yaw) = state.spatial.attitude.euler_angles();
        println!("Initial state:");
        println!("  pitch: {}", pitch);
        println!("  alpha: {}", state.air_data.alpha);
        println!("  q: {}", state.spatial.angular_velocity.y);
        println!("  forces: {:?}", state.physics.forces);
        println!("  moments: {:?}", state.physics.moments);
        pitch
    } else {
        panic!("Aircraft state not found");
    };

    // Apply elevator deflection
    if let Some(mut state) = app.query_single_mut::<FullAircraftState>() {
        let mut controls = state.control_surfaces;
        controls.elevator = 0.5; // 50% up elevator
        state.control_surfaces = controls;
        println!("Applied elevator deflection: {}", controls.elevator);
    }

    // Run simulation
    app.run_steps(120);

    if let Some(state) = app.query_single::<FullAircraftState>() {
        let (_roll, final_pitch, _yaw) = state.spatial.attitude.euler_angles();
        println!("Final state:");
        println!("  pitch: {}", final_pitch);
        println!("  alpha: {}", state.air_data.alpha);
        println!("  q: {}", state.spatial.angular_velocity.y);
        println!("  forces: {:?}", state.physics.forces);
        println!("  moments: {:?}", state.physics.moments);

        // Verify pitch response
        assert!(
            final_pitch > initial_pitch,
            "Elevator deflection should cause pitch up movement"
        );

        // Verify pitch rate
        assert!(
            state.spatial.angular_velocity.y > 0.0,
            "Positive elevator should create positive pitch rate"
        );

        // Verify angle of attack increase
        assert!(
            state.air_data.alpha > 0.0,
            "Elevator deflection should increase angle of attack"
        );

        // Check reasonable limits
        assert!(
            state.air_data.alpha < 15.0 * std::f64::consts::PI / 180.0,
            "Angle of attack should remain within reasonable limits"
        );
    }
}

#[test]
fn test_aileron_control() {
    let aircraft_config = create_test_full_config();
    let mut app = TestAppBuilder::new()
        .with_full_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    // Get initial roll attitude
    let initial_roll = if let Some(state) = app.query_single::<FullAircraftState>() {
        let (roll, _pitch, _yaw) = state.spatial.attitude.euler_angles();
        roll
    } else {
        panic!("Aircraft state not found");
    };

    // Apply aileron deflection
    if let Some(mut state) = app.query_single_mut::<FullAircraftState>() {
        let mut controls = neutral_controls();
        controls.aileron = 0.5; // 50% right aileron
        state.control_surfaces = controls;
    }

    // Run simulation for 1 second
    app.run_steps(120);

    if let Some(state) = app.query_single::<FullAircraftState>() {
        assert_full_state_valid(state);

        // Check roll angle has changed
        let (final_roll, _pitch, _yaw) = state.spatial.attitude.euler_angles();
        assert!(
            final_roll > initial_roll,
            "Right aileron should cause right roll"
        );

        // Verify roll rate is positive
        assert!(
            state.spatial.angular_velocity.x > 0.0,
            "Positive aileron should create positive roll rate"
        );
    }
}

#[test]
fn test_rudder_control() {
    let aircraft_config = create_test_full_config();
    let mut app = TestAppBuilder::new()
        .with_full_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    // Get initial heading
    let initial_heading = if let Some(state) = app.query_single::<FullAircraftState>() {
        let (_roll, _pitch, yaw) = state.spatial.attitude.euler_angles();
        yaw
    } else {
        panic!("Aircraft state not found");
    };

    // Apply rudder deflection
    if let Some(mut state) = app.query_single_mut::<FullAircraftState>() {
        let mut controls = neutral_controls();
        controls.rudder = 0.5; // 50% right rudder
        state.control_surfaces = controls;
    }

    // Run simulation for 2 seconds
    app.run_steps(240);

    if let Some(state) = app.query_single::<FullAircraftState>() {
        assert_full_state_valid(state);

        // Check heading has changed
        let (_roll, _pitch, final_heading) = state.spatial.attitude.euler_angles();
        assert!(
            final_heading > initial_heading,
            "Right rudder should cause right yaw"
        );

        // Verify yaw rate is positive
        assert!(
            state.spatial.angular_velocity.z > 0.0,
            "Positive rudder should create positive yaw rate"
        );

        // Check for sideslip (should be present with rudder input)
        let velocity = state.spatial.velocity;
        assert!(
            velocity.y != 0.0,
            "Rudder deflection should create sideslip"
        );
    }
}

#[test]
fn test_powerplant_control() {}

#[test]
fn test_coordinated_turn() {}

#[test]
fn test_climb() {}

#[test]
fn test_descent() {}

#[test]
fn test_stall_characteristics() {}

#[test]
fn test_speed_stability() {}

#[test]
fn test_control_surface_limits() {}

#[test]
fn test_flight_envelope_limits() {}
