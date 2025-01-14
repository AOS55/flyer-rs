use approx::assert_relative_eq;
use flyer::components::{
    AircraftState, DubinsAircraftState, FullAircraftState, PhysicsComponent, SpatialComponent,
};
use nalgebra::{UnitQuaternion, Vector3};

/// Assert that a spatial component's state is valid
#[track_caller]
pub fn assert_spatial_valid(spatial: &SpatialComponent) {
    // Verify position is finite
    assert!(spatial.position.x.is_finite(), "Position x is not finite");
    assert!(spatial.position.y.is_finite(), "Position y is not finite");
    assert!(spatial.position.z.is_finite(), "Position z is not finite");

    // Verify velocity is finite
    assert!(spatial.velocity.x.is_finite(), "Velocity x is not finite");
    assert!(spatial.velocity.y.is_finite(), "Velocity y is not finite");
    assert!(spatial.velocity.z.is_finite(), "Velocity z is not finite");

    // Verify angular velocity is finite
    assert!(
        spatial.angular_velocity.x.is_finite(),
        "Angular velocity x is not finite"
    );
    assert!(
        spatial.angular_velocity.y.is_finite(),
        "Angular velocity y is not finite"
    );
    assert!(
        spatial.angular_velocity.z.is_finite(),
        "Angular velocity z is not finite"
    );
}

/// Assert that a physics component's state is valid
#[track_caller]
pub fn assert_physics_valid(physics: &PhysicsComponent) {
    // Verify mass is positive and finite
    assert!(physics.mass > 0.0, "Mass must be positive");
    assert!(physics.mass.is_finite(), "Mass must be finite");

    // Verify inertia matrix is valid
    assert!(
        physics.inertia.iter().all(|x| x.is_finite()),
        "Inertia matrix contains non-finite values"
    );
    assert!(
        physics.inertia_inv.iter().all(|x| x.is_finite()),
        "Inverse inertia matrix contains non-finite values"
    );

    // Verify forces and moments
    assert!(
        physics.net_force.iter().all(|x| x.is_finite()),
        "Net force contains non-finite values"
    );
    assert!(
        physics.net_moment.iter().all(|x| x.is_finite()),
        "Net moment contains non-finite values"
    );
}

/// Assert that two positions are approximately equal
#[track_caller]
pub fn assert_position_eq(actual: &Vector3<f64>, expected: &Vector3<f64>, epsilon: f64) {
    assert_relative_eq!(
        actual.x,
        expected.x,
        epsilon = epsilon,
        max_relative = epsilon
    );
    assert_relative_eq!(
        actual.y,
        expected.y,
        epsilon = epsilon,
        max_relative = epsilon
    );
    assert_relative_eq!(
        actual.z,
        expected.z,
        epsilon = epsilon,
        max_relative = epsilon
    );
}

/// Assert that two attitudes are approximately equal
#[track_caller]
pub fn assert_attitude_eq(
    actual: &UnitQuaternion<f64>,
    expected: &UnitQuaternion<f64>,
    epsilon: f64,
) {
    // Compare using angle difference
    let diff = actual.inverse() * expected;
    let angle = diff.angle();
    assert!(
        angle < epsilon,
        "Attitude difference {} exceeds epsilon {}",
        angle,
        epsilon
    );
}

/// Assert that aircraft state is valid
#[track_caller]
pub fn assert_aircraft_state_valid(state: &AircraftState) {
    match state {
        AircraftState::Dubins(dubins) => assert_dubins_state_valid(dubins),
        AircraftState::Full(full) => assert_full_state_valid(full),
    }
}

/// Assert that Dubins aircraft state is valid
#[track_caller]
pub fn assert_dubins_state_valid(state: &DubinsAircraftState) {
    assert_spatial_valid(&state.spatial);

    // Verify controls are within reasonable bounds
    assert!(
        state.controls.acceleration.abs() < 100.0,
        "Unreasonable acceleration"
    );
    assert!(
        state.controls.bank_angle.abs() < std::f64::consts::PI,
        "Invalid bank angle"
    );
    assert!(
        state.controls.vertical_speed.abs() < 100.0,
        "Unreasonable vertical speed"
    );
}

/// Assert that full aircraft state is valid
// TOO: break out in to component checks
#[track_caller]
pub fn assert_full_state_valid(state: &FullAircraftState) {
    assert_spatial_valid(&state.spatial);
    assert_physics_valid(&state.physics);

    // Verify air data is valid
    assert!(state.air_data.true_airspeed >= 0.0, "Negative airspeed");
    assert!(state.air_data.density > 0.0, "Invalid air density");
    assert!(
        state.air_data.dynamic_pressure >= 0.0,
        "Negative dynamic pressure"
    );

    // Verify control surfaces are within limits (should be based on radian config)
    assert!(
        state.control_surfaces.elevator.abs() <= 1.0,
        "Invalid elevator deflection"
    );
    assert!(
        state.control_surfaces.aileron.abs() <= 1.0,
        "Invalid aileron deflection"
    );
    assert!(
        state.control_surfaces.rudder.abs() <= 1.0,
        "Invalid rudder deflection"
    );
    assert!(
        state.control_surfaces.power_lever >= 0.0 && state.control_surfaces.power_lever <= 1.0,
        "Invalid flap setting"
    );
}
