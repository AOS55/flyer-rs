use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::components::aircraft::{DubinsAircraftConfig, DubinsAircraftState};

/// System for simulating the dynamics of a Dubins aircraft.
///
/// This system updates the state of Dubins aircraft based on their current controls,
/// configuration parameters, and elapsed time. It calculates new positions, velocities,
/// and attitudes for each aircraft in the simulation.
pub fn dubins_aircraft_system(
    mut query: Query<(&mut DubinsAircraftState, &DubinsAircraftConfig)>,
    _time: Res<Time<Fixed>>,
) {
    let dt = 1.0 / 120.0;

    for (mut aircraft, config) in query.iter_mut() {
        update_aircraft(&mut aircraft, config, dt);
    }
}

/// Updates the state of a single Dubins aircraft.
///
/// # Arguments
/// * `state` - The mutable state of the aircraft to update.
/// * `config` - The configuration parameters defining the aircraft's limits and capabilities.
/// * `dt` - The time step (in seconds) over which to apply the update.
fn update_aircraft(state: &mut DubinsAircraftState, config: &DubinsAircraftConfig, dt: f64) {
    info!("Updating Aircraft State");

    let controls = &state.controls;
    let spatial = &mut state.spatial;

    // Update speed based on acceleration, clamped within the aircraft's speed limits
    let acceleration = controls.acceleration;
    let speed = spatial.velocity.norm();
    let new_speed = (speed + acceleration * dt).clamp(config.min_speed, config.max_speed);

    // Extract the current heading (yaw) from the aircraft's attitude
    let (_roll, _pitch, yaw) = spatial.attitude.euler_angles();

    // Get bank angle (φ) and calculate turn rate (c_φ * φ)
    let bank_angle = controls
        .bank_angle
        .clamp(-config.max_bank_angle, config.max_bank_angle);
    let turn_rate = (bank_angle / config.max_bank_angle) * config.max_turn_rate; // c_φ * φ

    // Calculate pitch angle based on the vertical speed and maximum climb rate
    let pitch_angle = if config.max_climb_rate != 0.0 {
        (controls.vertical_speed / config.max_climb_rate) * (std::f64::consts::PI / 9.0)
    // Max ±20 degrees
    } else {
        0.0
    };

    info!(
        "position before update: {:?}, controls: {:?}",
        spatial.position, controls
    );
    // Update position based on speed, heading, and vertical speed
    spatial.position.x += new_speed * yaw.cos() * dt;
    spatial.position.y += new_speed * yaw.sin() * dt;
    spatial.position.z -= controls.vertical_speed * dt;
    info!("position after update: {:?}", spatial.position);

    // Update heading: θ_t+1 = θ_t + c_φ*φ*dt
    let heading_change = turn_rate * dt;

    // Create the new rotation based on updated heading and bank angle
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
