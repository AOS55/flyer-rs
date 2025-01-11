use approx::assert_relative_eq;
use flyer::{components::DubinsAircraftState, resources::PhysicsConfig};

use crate::common::{assert_dubins_state_valid, create_test_dubins_config, TestAppBuilder};

#[test]
fn test_straight_level_flight() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    // Run for 1 second of simulation time
    app.run_steps(120);

    if let Some(state) = app.query_single::<DubinsAircraftState>() {
        assert_dubins_state_valid(state);

        // Verify altitude is maintained
        let initial_altitude = -1000.0;
        assert_relative_eq!(state.spatial.position.z, initial_altitude, epsilon = 1.0);

        // Verify forward motion
        assert!(state.spatial.velocity.x > 0.0);
    } else {
        panic!("Aircraft state not found");
    }
}

#[test]
fn test_basic_turn() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    // Apply a bank angle for turning
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.bank_angle = 0.2; // Small bank angle
    }

    // Run for 2 seconds of simulation time
    app.run_steps(240);

    if let Some(state) = app.query_single::<DubinsAircraftState>() {
        assert_dubins_state_valid(state);

        // Verify turn is happening (heading has changed)
        let (_roll, _pitch, yaw) = state.spatial.attitude.euler_angles();
        assert!(yaw.abs() > 0.0);

        // Verify reasonable bank angle is maintained
        assert!(state.controls.bank_angle.abs() < std::f64::consts::PI / 4.0);
    } else {
        panic!("Aircraft state not found");
    }
}

#[test]
fn test_basic_climb() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    // Set vertical speed for climbing
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.vertical_speed = 5.0; // 5 m/s climb rate
    }

    // Run for 1 second of simulation time
    app.run_steps(120);

    if let Some(state) = app.query_single::<DubinsAircraftState>() {
        assert_dubins_state_valid(state);

        // Verify altitude has increased
        assert!(state.spatial.position.z < -1000.0);

        // Verify climb rate is maintained
        assert!(state.spatial.velocity.z < 0.0);
    } else {
        panic!("Aircraft state not found");
    }
}

#[test]
fn test_acceleration_deceleration() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    let initial_speed = if let Some(state) = app.query_single::<DubinsAircraftState>() {
        state.spatial.velocity.norm()
    } else {
        panic!("Aircraft state not found");
    };

    // Apply acceleration
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.acceleration = 2.0;
    }

    // Run for 1 second of simulation time
    app.run_steps(120);

    let final_speed = if let Some(state) = app.query_single::<DubinsAircraftState>() {
        assert_dubins_state_valid(state);
        state.spatial.velocity.norm()
    } else {
        panic!("Aircraft state not found");
    };

    // Verify speed has increased
    assert!(final_speed > initial_speed);
}
