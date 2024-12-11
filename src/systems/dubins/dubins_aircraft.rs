use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::components::aircraft::{DubinsAircraftConfig, DubinsAircraftState};

pub fn dubins_aircraft_system(
    mut query: Query<(&mut DubinsAircraftState, &DubinsAircraftConfig)>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs_f64();

    for (mut aircraft, config) in query.iter_mut() {
        update_aircraft(&mut aircraft, config, dt);
    }
}
fn update_aircraft(state: &mut DubinsAircraftState, config: &DubinsAircraftConfig, dt: f64) {
    let controls = &state.controls;
    let spatial = &mut state.spatial;

    // Update velocity magnitude based on acceleration
    let acceleration = controls.acceleration;
    let speed = spatial.velocity.norm();
    let new_speed = (speed + acceleration * dt).clamp(config.min_speed, config.max_speed);

    // Get current heading from attitude (θ)
    let (_roll, _pitch, yaw) = spatial.attitude.euler_angles();

    // Get bank angle (φ) and calculate turn rate (c_φ * φ)
    let bank_angle = controls
        .bank_angle
        .clamp(-config.max_bank_angle, config.max_bank_angle);
    let turn_rate = (bank_angle / config.max_bank_angle) * config.max_turn_rate; // c_φ * φ

    // Calculate pitch angle based on vertical speed ratio
    let pitch_angle = if config.max_climb_rate != 0.0 {
        (controls.vertical_speed / config.max_climb_rate) * (std::f64::consts::PI / 9.0)
    // Max ±20 degrees
    } else {
        0.0
    };

    // Update position
    spatial.position.x += new_speed * yaw.cos() * dt;
    spatial.position.y += new_speed * yaw.sin() * dt;
    spatial.position.z -= controls.vertical_speed * dt;

    // Update heading: θ_t+1 = θ_t + c_φ*φ*dt
    let heading_change = turn_rate * dt;

    // Create rotation from bank and heading
    // First apply heading change around Z axis
    let heading_rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, yaw + heading_change);
    // Then apply bank around X axis (body roll)
    let bank_rotation = UnitQuaternion::from_euler_angles(bank_angle, pitch_angle, 0.0);
    spatial.attitude = heading_rotation * bank_rotation;

    // Update velocity vector to match new heading and speed
    spatial.velocity = Vector3::new(
        new_speed * yaw.cos(),
        new_speed * yaw.sin(),
        controls.vertical_speed,
    );
}
