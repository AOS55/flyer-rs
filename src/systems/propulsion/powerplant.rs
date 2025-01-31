use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    components::{
        AirData, AircraftControlSurfaces, Force, ForceCategory, FullAircraftConfig,
        PhysicsComponent, PowerplantConfig, PowerplantState, PropulsionState, ReferenceFrame,
    },
    resources::PhysicsConfig,
};

/// System for calculating propulsion forces and moments
pub fn propulsion_system(
    mut query: Query<(
        &AircraftControlSurfaces,
        &mut PropulsionState,
        &mut PhysicsComponent,
        &AirData,
        &FullAircraftConfig,
    )>,
    physics_config: Res<PhysicsConfig>,
) {
    let dt = physics_config.timestep;
    for (controls, mut propulsion_state, mut physics, air_data, aircraft_config) in query.iter_mut()
    {
        // Remove any existing propulsive forces before adding new ones
        physics
            .forces
            .retain(|force| force.category != ForceCategory::Propulsive);

        physics
            .moments
            .retain(|moment| moment.category != ForceCategory::Propulsive);

        propulsion_state.set_power_lever(controls.power_lever);

        // Calculate all engine updates and store them in a temporary vector
        let engine_updates: Vec<(PowerplantState, Force)> = aircraft_config
            .propulsion
            .engines
            .iter()
            .zip(propulsion_state.engine_states.iter())
            .map(|(engine_config, engine_state)| {
                // Create a mutable copy of the engine state
                let mut updated_state = engine_state.clone();

                // Update engine dynamics
                update_powerplant_state(&mut updated_state, engine_config, dt);

                // Calculate thrust and fuel flow
                let (thrust, fuel_flow) = calculate_thrust(
                    engine_config,
                    &updated_state,
                    air_data.density,
                    air_data.true_airspeed,
                );

                // Update fuel flow in the state
                updated_state.fuel_flow = fuel_flow;

                // Create thrust vector in body frame
                let thrust_direction = UnitQuaternion::from_euler_angles(
                    engine_config.orientation.x,
                    engine_config.orientation.y,
                    engine_config.orientation.z,
                ) * Vector3::new(1.0, 0.0, 0.0);

                let force_vector = thrust_direction * thrust;

                // Create force
                let force = Force {
                    vector: force_vector,
                    point: Some(engine_config.position),
                    frame: ReferenceFrame::Body,
                    category: ForceCategory::Propulsive,
                };

                println!(
                    "engine_config: {:?}, engine_state {:?}",
                    engine_config, updated_state
                );

                (updated_state, force)
            })
            .collect();

        // Apply all updates at once
        for (idx, (updated_state, force)) in engine_updates.into_iter().enumerate() {
            propulsion_state.engine_states[idx] = updated_state;
            physics.add_force(force);
        }
    }
}

/// Updates the engine state based on throttle setting and time constants
fn update_powerplant_state(state: &mut PowerplantState, config: &PowerplantConfig, dt: f64) {
    if !state.running && state.power_lever > 0.0 {
        state.running = true;
    } else if state.power_lever <= 0.0 {
        state.running = false;
    }

    let target_thrust = if state.running {
        state.power_lever
    } else {
        0.0
    };

    // Determine appropriate time constant based on whether we're spooling up or down
    let time_constant = if target_thrust > state.thrust_fraction {
        config.spool_up_time
    } else {
        config.spool_down_time
    };

    // First-order response for engine dynamics
    if time_constant > 0.0 {
        let alpha = dt / time_constant;
        state.thrust_fraction += alpha * (target_thrust - state.thrust_fraction);
    } else {
        state.thrust_fraction = target_thrust;
    }

    // Ensure thrust fraction stays within bounds
    state.thrust_fraction = state.thrust_fraction.clamp(0.0, 1.0);
}

/// Calculates thrust and fuel flow based on current conditions
fn calculate_thrust(
    config: &PowerplantConfig,
    state: &PowerplantState,
    air_density: f64,
    airspeed: f64,
) -> (f64, f64) {
    // Atmospheric correction factor (simplified)
    let rho_ratio = air_density / 1.225; // Relative to sea level density
    let rho_factor = rho_ratio.sqrt(); // Simplified density correction

    // Ram drag factor (simplified)
    let mach = airspeed / 340.0; // Approximate Mach number at sea level
    let ram_factor = 1.0 - 0.1 * mach; // Simple linear reduction with Mach

    // Calculate base thrust
    let max_thrust_available = config.max_thrust * rho_factor * ram_factor;
    let min_thrust_available = config.min_thrust * rho_factor;

    let thrust = min_thrust_available
        + (max_thrust_available - min_thrust_available) * state.thrust_fraction;

    // Calculate fuel flow
    let fuel_flow = if state.running {
        thrust * config.tsfc * (1.0 + 0.2 * state.thrust_fraction) // Add inefficiency at high power
    } else {
        0.0
    };

    (thrust, fuel_flow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_engine_spool_up() {
        let config = PowerplantConfig::default();
        let mut state = PowerplantState::default();

        // Set throttle to max
        state.power_lever = 1.0;

        // Update for one second
        update_powerplant_state(&mut state, &config, 1.0);

        // Check that thrust is increasing but not immediately at max
        assert!(state.thrust_fraction > 0.0);
        assert!(state.thrust_fraction < 1.0);
        assert!(state.running);
    }

    #[test]
    fn test_thrust_calculation() {
        let config = PowerplantConfig::default();
        let mut state = PowerplantState::default();
        state.thrust_fraction = 1.0;
        state.running = true;

        // Test at sea level, zero airspeed
        let (thrust, fuel_flow) = calculate_thrust(&config, &state, 1.225, 0.0);
        assert_relative_eq!(thrust, config.max_thrust, epsilon = 1e-10);
        assert!(fuel_flow > 0.0);

        // Test at altitude (lower density)
        let (thrust_altitude, _) = calculate_thrust(&config, &state, 0.5, 0.0);
        assert!(thrust_altitude < thrust);

        // Test with airspeed
        let (thrust_speed, _) = calculate_thrust(&config, &state, 1.225, 100.0);
        assert!(thrust_speed < thrust);
    }
}
