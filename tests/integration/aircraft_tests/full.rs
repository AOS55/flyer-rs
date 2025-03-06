use approx::assert_relative_eq;
use bevy::prelude::*;
use flyer::{
    components::{
        AirData, AircraftControlSurfaces, NeedsTrim, PhysicsComponent, PropulsionState,
        SpatialComponent, TrimCondition, TrimStage,
    },
    resources::PhysicsConfig,
};
use nalgebra::Vector3;

use crate::common::{create_test_full_config, wait_for_condition, TestAppBuilder};

#[test]
fn test_straight_level_flight() {
    let aircraft_config = create_test_full_config();
    let mut app = TestAppBuilder::new()
        .with_full_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();

    if let Some(mut propulsion) = app.query_single_mut::<PropulsionState>() {
        propulsion.set_power_lever(0.5);
        propulsion.turn_engines_on();
    }

    // Run for 0.05 seconds of simulation time
    app.run_steps(6);

    if let Some(spatial) = app.query_single::<SpatialComponent>() {
        // Verify altitude is maintained
        let initial_altitude = -600.0;
        assert_relative_eq!(spatial.position.z, initial_altitude, epsilon = 1.0);

        // Verify forward motion
        assert!(spatial.velocity.x > 0.0);
    } else {
        panic!("Aircraft state not found");
    }

    // Add trim request for S+L flight at 100 m/s
    let trimmed = wait_for_condition(
        &mut app,
        |app| {
            let world = app.world_mut();

            let entity = world
                .query_filtered::<Entity, With<AircraftControlSurfaces>>()
                .get_single(world)
                .ok();

            if let Some(entity) = entity {
                world.entity_mut(entity).insert(NeedsTrim {
                    condition: TrimCondition::StraightAndLevel { airspeed: 100.0 },
                    solver: None,
                    stage: TrimStage::Longitudinal,
                });
                true
            } else {
                false
            }
        },
        10,
    );

    assert!(trimmed, "Failed to request trim");

    // Run simulation until trim converges (max 1000 steps)
    let trimmed = wait_for_condition(
        &mut app,
        |app| {
            app.world_mut()
                .query_filtered::<Entity, With<NeedsTrim>>()
                .iter(&app.world())
                .next()
                .is_none()
        },
        1000,
    );

    assert!(trimmed, "Trim did not converge");

    // Get aircraft state and validate
    if let Some(spatial) = app.query_single::<SpatialComponent>() {
        // Verify straight and level conditions
        let velocity = spatial.velocity;
        let airspeed = velocity.norm();
        println!("velocity: {}, airspeed: {}", velocity, airspeed);
        assert!((airspeed - 50.0).abs() < 1.0, "Airspeed not at target");

        // Check vertical speed is close to zero
        assert!(velocity.z.abs() < 0.1, "Significant vertical speed present");

        // Check roll and pitch angles are small
        let (roll, pitch, _) = spatial.attitude.euler_angles();
        assert!(roll.abs() < 0.01, "Significant roll angle present");
        assert!(pitch.abs() < 0.1, "Significant pitch angle present");
    } else {
        panic!("Could not find aircraft state");
    }

    // Run for 10 seconds and verify stable flight
    app.run_steps((10.0 / app.steps_per_action as f64) as usize);

    if let Some(spatial) = app.query_single::<SpatialComponent>() {
        let velocity = spatial.velocity;
        let airspeed = velocity.norm();
        assert!(
            (airspeed - 100.0).abs() < 1.0,
            "Failed to maintain airspeed"
        );
        assert!(velocity.z.abs() < 0.1, "Failed to maintain altitude");
    } else {
        panic!("Could not find final aircraft state");
    }
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

    if let Some(mut propulsion) = app.query_single_mut::<PropulsionState>() {
        propulsion.set_power_lever(0.5);
        propulsion.turn_engines_on();
    }

    // Initialize with steady flight conditions
    if let Some((mut spatial, mut control_surfaces, physics)) =
        app.query_tuple3_single_mut::<SpatialComponent, AircraftControlSurfaces, PhysicsComponent>()
    {
        // Set initial conditions for stable flight
        spatial.velocity = Vector3::new(50.0, 0.0, 0.0);

        // Print initial forces and moments
        println!("Initial forces: {:?}", physics.forces);
        println!("Initial moments: {:?}", physics.moments);

        // Set trim conditions
        control_surfaces.elevator = 0.0;
        control_surfaces.aileron = 0.0;
        control_surfaces.rudder = 0.0;
        control_surfaces.power_lever = 0.5; // Add power to maintain flight
    }

    // Run for a short time to stabilize
    app.run_steps(60);

    // Get initial pitch attitude
    let initial_pitch = if let Some((spatial, air_data, physics)) =
        app.query_tuple3_single_mut::<SpatialComponent, AirData, PhysicsComponent>()
    {
        let (_roll, pitch, _yaw) = spatial.attitude.euler_angles();
        println!("Initial state:");
        println!("  pitch: {}", pitch);
        println!("  alpha: {}", air_data.alpha);
        println!("  q: {}", spatial.angular_velocity.y);
        println!("  forces: {:?}", physics.forces);
        println!("  moments: {:?}", physics.moments);
        pitch
    } else {
        panic!("Aircraft state not found");
    };

    // Apply elevator deflection
    if let Some(mut control_surfaces) = app.query_single_mut::<AircraftControlSurfaces>() {
        control_surfaces.elevator = 0.1;
        println!("Applied elevator deflection: {}", control_surfaces.elevator);
    }

    // Run simulation
    app.run_steps(120);

    if let Some((spatial, air_data, physics)) =
        app.query_tuple3_single_mut::<SpatialComponent, AirData, PhysicsComponent>()
    {
        let (_roll, final_pitch, _yaw) = spatial.attitude.euler_angles();
        println!("Final state:");
        println!("  pitch: {}", final_pitch);
        println!("  alpha: {}", air_data.alpha);
        println!("  q: {}", spatial.angular_velocity.y);
        println!("  forces: {:?}", physics.forces);
        println!("  moments: {:?}", physics.moments);

        // Verify pitch response
        assert!(
            final_pitch > initial_pitch,
            "Elevator deflection should cause pitch up movement"
        );

        // Verify pitch rate
        assert!(
            spatial.angular_velocity.y > 0.0,
            "Positive elevator should create positive pitch rate"
        );

        // Verify angle of attack increase
        assert!(
            air_data.alpha > 0.0,
            "Elevator deflection should increase angle of attack"
        );

        // Check reasonable limits
        assert!(
            air_data.alpha < 15.0 * std::f64::consts::PI / 180.0,
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

    if let Some(mut propulsion) = app.query_single_mut::<PropulsionState>() {
        propulsion.set_power_lever(0.5);
        propulsion.turn_engines_on();
    }

    // Initialize with steady flight conditions
    if let Some((mut spatial, mut control_surfaces, physics)) =
        app.query_tuple3_single_mut::<SpatialComponent, AircraftControlSurfaces, PhysicsComponent>()
    {
        // Set initial conditions for stable flight
        spatial.velocity = Vector3::new(50.0, 0.0, 0.0);

        // Print initial forces and moments
        println!("Initial forces: {:?}", physics.forces);
        println!("Initial moments: {:?}", physics.moments);

        // Set trim conditions
        control_surfaces.elevator = 0.0;
        control_surfaces.aileron = 0.0;
        control_surfaces.rudder = 0.0;
        control_surfaces.power_lever = 0.5; // Add power to maintain flight
    }

    // Run for a short time to stabilize
    app.run_steps(60);

    // Get initial roll attitude
    let initial_roll = if let Some((spatial, air_data, physics)) =
        app.query_tuple3_single_mut::<SpatialComponent, AirData, PhysicsComponent>()
    {
        let (roll, _pitch, _yaw) = spatial.attitude.euler_angles();
        println!("Initial state:");
        println!("  roll: {}", roll);
        println!("  alpha: {}", air_data.alpha);
        println!("  p: {}", spatial.angular_velocity.x);
        println!("  forces: {:?}", physics.forces);
        println!("  moments: {:?}", physics.moments);
        roll
    } else {
        panic!("Aircraft state not found");
    };

    // Apply aileron deflection
    if let Some(mut control_surfaces) = app.query_single_mut::<AircraftControlSurfaces>() {
        control_surfaces.aileron = 0.5;
    }

    // Run simulation
    app.run_steps(120);

    if let Some(spatial) = app.query_single::<SpatialComponent>() {
        // Check roll angle has changed
        let (final_roll, _pitch, _yaw) = spatial.attitude.euler_angles();
        assert!(
            final_roll > initial_roll,
            "Right aileron should cause right roll"
        );

        // Verify roll rate is positive
        assert!(
            spatial.angular_velocity.x > 0.0,
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

    if let Some(mut propulsion) = app.query_single_mut::<PropulsionState>() {
        propulsion.set_power_lever(0.5);
        propulsion.turn_engines_on();
    }

    // Initialize with steady flight conditions
    if let Some((mut spatial, mut control_surfaces, physics)) =
        app.query_tuple3_single_mut::<SpatialComponent, AircraftControlSurfaces, PhysicsComponent>()
    {
        // Set initial conditions for stable flight
        spatial.velocity = Vector3::new(50.0, 0.0, 0.0);

        // Print initial forces and moments
        println!("Initial forces: {:?}", physics.forces);
        println!("Initial moments: {:?}", physics.moments);

        // Set trim conditions
        control_surfaces.elevator = 0.0;
        control_surfaces.aileron = 0.0;
        control_surfaces.rudder = 0.0;
        control_surfaces.power_lever = 0.5; // Add power to maintain flight
    }

    app.run_steps(60);

    // Get initial heading
    let initial_heading = if let Some((spatial, air_data, physics)) =
        app.query_tuple3_single_mut::<SpatialComponent, AirData, PhysicsComponent>()
    {
        let (_roll, _pitch, yaw) = spatial.attitude.euler_angles();
        println!("Initial state:");
        println!("  yaw: {}", yaw);
        println!("  alpha: {}", air_data.alpha);
        println!("  r: {}", spatial.angular_velocity.z);
        println!("  forces: {:?}", physics.forces);
        println!("  moments: {:?}", physics.moments);
        yaw
    } else {
        panic!("Aircraft state not found");
    };

    // Apply rudder deflection
    if let Some(mut control_surfaces) = app.query_single_mut::<AircraftControlSurfaces>() {
        control_surfaces.rudder = 0.5;
    }

    // Run simulation for 2 seconds
    app.run_steps(240);

    if let Some(spatial) = app.query_single::<SpatialComponent>() {
        // Check heading has changed
        let (_roll, _pitch, final_heading) = spatial.attitude.euler_angles();
        assert!(
            final_heading > initial_heading,
            "Right rudder should cause right yaw"
        );

        // Verify yaw rate is positive
        assert!(
            spatial.angular_velocity.z > 0.0,
            "Positive rudder should create positive yaw rate"
        );

        // Check for sideslip (should be present with rudder input)
        let velocity = spatial.velocity;
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

#[test]
fn test_trim_convergence() {
    let aircraft_config = create_test_full_config();
    let mut app = TestAppBuilder::new()
        .with_full_aircraft(aircraft_config)
        .with_physics(PhysicsConfig::default())
        .build();
    
    // Set initial conditions
    if let Some(mut spatial) = app.query_single_mut::<SpatialComponent>() {
        spatial.velocity = Vector3::new(50.0, 0.0, 0.0);
    }
    
    if let Some(mut propulsion) = app.query_single_mut::<PropulsionState>() {
        propulsion.set_power_lever(0.5);
        propulsion.turn_engines_on();
    }
    
    // Add trim request for straight and level flight
    let trimmed = wait_for_condition(
        &mut app,
        |app| {
            let world = app.world_mut();
            
            let entity = world
                .query_filtered::<Entity, With<AircraftControlSurfaces>>()
                .get_single(world)
                .ok();
                
            if let Some(entity) = entity {
                world.entity_mut(entity).insert(NeedsTrim {
                    condition: TrimCondition::StraightAndLevel { airspeed: 70.0 },
                    solver: None,
                    stage: TrimStage::Longitudinal,
                });
                true
            } else {
                false
            }
        },
        10,
    );
    
    assert!(trimmed, "Failed to request trim");
    
    // Run for up to 1000 steps until trim converges
    let trimmed = wait_for_condition(
        &mut app,
        |app| {
            // Check if NeedsTrim component has been removed (trim complete)
            app.world_mut()
                .query_filtered::<Entity, With<NeedsTrim>>()
                .iter(&app.world())
                .next()
                .is_none()
        },
        1000,
    );
    
    assert!(trimmed, "Trim did not converge within 1000 steps");
    
    // Verify the aircraft is actually trimmed by checking if forces and moments are balanced
    // Get the initial state before any further calculations
    let (spatial_initial, controls_initial) = {
        let result = app.query_tuple3_single::<SpatialComponent, AircraftControlSurfaces, PhysicsComponent>();
        if let Some((spatial, controls, _)) = result {
            // Print final state
            let (roll, pitch, _yaw) = spatial.attitude.euler_angles();
            println!("Final trim state:");
            println!("  Airspeed: {:.1} m/s", spatial.velocity.norm());
            println!("  Pitch angle: {:.2}° (radians: {:.4})", pitch.to_degrees(), pitch);
            println!("  Roll angle: {:.2}° (radians: {:.4})", roll.to_degrees(), roll);
            println!("  Elevator position: {:.4}", controls.elevator);
            println!("  Throttle position: {:.4}", controls.power_lever);
            
            // Store important values for later comparison
            (spatial.clone(), controls.clone())
        } else {
            panic!("Failed to get initial aircraft state");
        }
    };
    
    // Run one more simulation step to calculate forces
    app.run_steps(1);
    
    // Now get the physics state after force calculation
    let (force_magnitude, moment_magnitude, mass) = {
        if let Some(physics) = app.query_single::<PhysicsComponent>() {
            // In trimmed flight, net forces and moments should be small
            let force_mag = physics.net_force.norm();
            let moment_mag = physics.net_moment.norm();
            
            println!("  Net force magnitude: {:.2} N", force_mag);
            println!("  Net moment magnitude: {:.2} N·m", moment_mag);
            
            (force_mag, moment_mag, physics.mass)
        } else {
            panic!("Failed to get physics component after force calculation");
        }
    };
    
    // Calculate weight for scaling
    let weight = mass * 9.81; // Approximate weight
    
    // Assert that forces and moments are reasonably balanced
    // For trim, moments should be very small relative to aircraft weight
    assert!(
        moment_magnitude < weight * 0.05, // Moments should be less than 5% of weight
        "Moments not properly balanced in trimmed flight: {:.2} N·m", 
        moment_magnitude
    );
    
    // Verify airspeed is close to target
    let target_speed = 70.0;
    let initial_airspeed = spatial_initial.velocity.norm();
    let airspeed_error = (initial_airspeed - target_speed).abs() / target_speed;
    assert!(
        airspeed_error < 0.1, // Within 10% of target speed
        "Airspeed not maintained: {:.1} m/s vs target {:.1} m/s", 
        initial_airspeed, 
        target_speed
    );
    
    // Test trim stability by running for 5 more seconds
    app.run_steps(500);
    
    // Check if aircraft maintains trim condition
    if let Some(final_spatial) = app.query_single::<SpatialComponent>() {
        let final_airspeed = final_spatial.velocity.norm();
        let speed_variance = (final_airspeed - initial_airspeed).abs();
        
        println!("After 5 seconds:");
        println!("  Airspeed: {:.1} m/s (change: {:.1} m/s)", 
            final_airspeed, speed_variance);
        
        let (_, final_pitch, _) = final_spatial.attitude.euler_angles();
        let (_, initial_pitch, _) = spatial_initial.attitude.euler_angles();
        let pitch_variance = (final_pitch - initial_pitch).abs().to_degrees();
        
        println!("  Pitch: {:.2}° (change: {:.2}°)", 
            final_pitch.to_degrees(), pitch_variance);
        
        // Check stability criteria
        assert!(
            speed_variance < 5.0,
            "Trimmed aircraft failed to maintain stable airspeed"
        );
        
        assert!(
            pitch_variance < 5.0,
            "Trimmed aircraft failed to maintain stable pitch attitude"
        );
    } else {
        panic!("Aircraft components not found after trim");
    }
}
