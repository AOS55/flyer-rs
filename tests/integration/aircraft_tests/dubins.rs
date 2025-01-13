use approx::assert_relative_eq;
use bevy::prelude::*;
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
        let initial_altitude = -500.0;
        assert_relative_eq!(state.spatial.position.z, initial_altitude, epsilon = 1.0);

        // Verify forward motion
        assert!(state.spatial.velocity.x > 0.0);
    } else {
        panic!("Aircraft state not found");
    }
}

#[test]
fn test_turn() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();
    app.run_frame();

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
fn test_climb() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();
    app.run_frame();

    let initial_state = app.query_single_mut::<DubinsAircraftState>();
    info!("initial_state: {:?}", initial_state);
    assert!(
        initial_state.is_some(),
        "Failed to get initial aircraft state"
    );

    // Set vertical speed for climbing
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.vertical_speed = 5.0; // 5 m/s climb rate
        println!("initial state: {:?}", state);
    }

    // Run for 1 second of simulation time
    app.run_steps(120);

    if let Some(state) = app.query_single::<DubinsAircraftState>() {
        println!("post state: {:?}", state);

        assert_dubins_state_valid(state);

        // Verify altitude has increased
        assert!(state.spatial.position.z < -500.0);

        // Verify climb rate is maintained
        assert!(state.spatial.velocity.z > 0.0);
    } else {
        panic!("Aircraft state not found");
    }
}

#[test]
fn test_descent() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();
    app.run_frame();

    // Set vertical speed for descending
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.vertical_speed = -5.0; // -5 m/s descent rate
    }

    app.run_steps(120); // Run for 1 second

    if let Some(state) = app.query_single::<DubinsAircraftState>() {
        assert_dubins_state_valid(state);
        assert!(state.spatial.position.z > -500.0); // Verify altitude has decreased
        assert!(state.spatial.velocity.z < 0.0); // Verify descent rate
    }
}

#[test]
fn test_acceleration() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();
    app.run_frame();

    let initial_speed = if let Some(state) = app.query_single_mut::<DubinsAircraftState>() {
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

    let final_speed = if let Some(state) = app.query_single_mut::<DubinsAircraftState>() {
        assert_dubins_state_valid(&state);
        state.spatial.velocity.norm()
    } else {
        panic!("Aircraft state not found");
    };

    // Verify speed has increased
    assert!(final_speed > initial_speed);
}

#[test]
fn test_deceleration() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();
    app.run_frame();

    let initial_speed = if let Some(state) = app.query_single::<DubinsAircraftState>() {
        println!("initial speed: {:?}", state);
        state.spatial.velocity.norm()
    } else {
        panic!("Aircraft state not found");
    };

    // Apply deceleration
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.acceleration = -2.0;
    }

    app.run_steps(120);

    let final_speed = if let Some(state) = app.query_single_mut::<DubinsAircraftState>() {
        assert_dubins_state_valid(&state);
        state.spatial.velocity.norm()
    } else {
        panic!("Aircraft state not found");
    };

    println!(
        "initial speed: {}, final speed: {}",
        initial_speed, final_speed
    );

    assert!(final_speed < initial_speed);
}

#[test]
fn test_climbing_turn() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();
    app.run_frame();

    // Set both vertical speed and bank angle
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.vertical_speed = 3.0;
        state.controls.bank_angle = 0.2;
    }

    app.run_steps(240);

    if let Some(state) = app.query_single::<DubinsAircraftState>() {
        assert_dubins_state_valid(state);
        assert!(state.spatial.position.z < -500.0); // Verify climbing
        let (_roll, _pitch, yaw) = state.spatial.attitude.euler_angles();
        assert!(yaw.abs() > 0.0); // Verify turning
    }
}

#[test]
fn test_speed_limits() {
    let mut aircraft_config = create_test_dubins_config();
    aircraft_config.max_speed = 100.0;
    aircraft_config.min_speed = 20.0;

    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config.clone())
        .with_physics(PhysicsConfig::default())
        .build();
    app.run_frame();

    // Try to accelerate beyond max speed
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.acceleration = 10.0;
    }

    app.run_steps(600); // Run for 5 seconds

    if let Some(state) = app.query_single::<DubinsAircraftState>() {
        assert_dubins_state_valid(state);
        assert!(state.spatial.velocity.norm() <= aircraft_config.max_speed);
    }
}

#[test]
fn test_control_limits() {
    let aircraft_config = create_test_dubins_config();
    let mut app = TestAppBuilder::new()
        .with_dubins_aircraft(aircraft_config.clone())
        .with_physics(PhysicsConfig::default())
        .build();
    app.run_frame();

    // Try to exceed maximum bank angle and vertical speed
    if let Some(mut state) = app.query_single_mut::<DubinsAircraftState>() {
        state.controls.bank_angle = 2.0 * aircraft_config.max_bank_angle;
        state.controls.vertical_speed = 2.0 * aircraft_config.max_climb_rate;
    }

    app.run_steps(120);

    if let Some(state) = app.query_single::<DubinsAircraftState>() {
        assert_dubins_state_valid(state);
        println!(
            "bank_angle: {:?}, lim: {:?}",
            state.controls.bank_angle.abs(),
            aircraft_config.max_bank_angle,
        );
        assert!(state.controls.bank_angle.abs() <= aircraft_config.max_bank_angle);
        assert!(state.controls.vertical_speed <= aircraft_config.max_climb_rate);
    }
}
