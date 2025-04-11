// src/systems/propulsion/powerplant.rs

use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    components::{
        AirData,
        AircraftControlSurfaces,
        Force,
        ForceCategory,
        FullAircraftConfig,
        // Moment, // Removed unused import
        PhysicsComponent,
        PowerplantConfig,
        PowerplantState,
        PropulsionState,
        ReferenceFrame,
        // SpatialComponent, // Removed unused import
    },
    // Assuming AirDataValues is defined elsewhere (e.g., air_data.rs or calculate.rs)
    // aerodynamics::air_data::AirDataValues,
    resources::PhysicsConfig,
};

// Placeholder for AirDataValues if not imported from elsewhere
#[derive(Debug, Clone, Default)]
pub struct AirDataValues {
    pub true_airspeed: f64,
    pub alpha: f64,
    pub beta: f64,
    pub density: f64,
    pub dynamic_pressure: f64,
    pub relative_velocity_body: Vector3<f64>,
}

// --- Helper Functions (Keep or move to calculate.rs) ---

/// Updates the engine state based on throttle setting and time constants.
/// This function MUTATES the state based on time step 'dt'.
pub fn update_powerplant_state(state: &mut PowerplantState, config: &PowerplantConfig, dt: f64) {
    // Determine if engine should be running based on power lever
    // (Consider adding a separate 'engine_enabled' state if needed)
    if !state.running && state.power_lever > 0.01 {
        // Use a small threshold to start
        state.running = true;
    } else if state.running && state.power_lever <= 0.0 {
        // Only set running to false if it was running
        state.running = false;
        // Let thrust spool down naturally unless immediate cut-off is desired
    }

    let target_thrust_fraction = if state.running {
        state.power_lever // Target fraction is the lever position when running
    } else {
        0.0 // Target is zero when off
    };

    // Determine appropriate time constant based on whether we're spooling up or down
    let time_constant = if target_thrust_fraction > state.thrust_fraction {
        config.spool_up_time
    } else {
        config.spool_down_time
    };

    // First-order response for engine dynamics (spooling)
    if time_constant > 1e-6 {
        // Avoid division by zero or near-zero
        // Using exponential decay: s(t+dt) = target + (s(t) - target) * exp(-dt/T)
        let alpha = (-dt / time_constant).exp(); // Decay factor
        state.thrust_fraction =
            target_thrust_fraction + (state.thrust_fraction - target_thrust_fraction) * alpha;
    } else {
        // If time constant is zero or negative, snap directly to target
        state.thrust_fraction = target_thrust_fraction;
    }

    // Ensure thrust fraction stays within valid bounds [0, 1]
    state.thrust_fraction = state.thrust_fraction.clamp(0.0, 1.0);
}

/// Calculates thrust scalar and fuel flow scalar based on current conditions.
/// This function is PURE - calculation based on inputs.
fn calculate_thrust_and_fuel_flow(
    config: &PowerplantConfig,
    state: &PowerplantState, // Takes the *current* state (thrust_fraction)
    air_density: f64,
    airspeed: f64,
) -> (f64, f64) {
    // Returns (thrust_scalar, fuel_flow_scalar)
    // Atmospheric correction factor (simplified model)
    let rho_sealevel = 1.225;
    let rho_ratio = (air_density / rho_sealevel).max(0.01); // Avoid zero/negative density ratio
                                                            // *** CORRECTION: Use sqrt() as exponent field doesn't exist ***
    let rho_factor = rho_ratio.sqrt(); // Simple density correction (power 0.5)

    // Ram drag / Mach effect factor (simplified model)
    let speed_of_sound_sea_level = 340.3; // m/s approx
    let mach = (airspeed / speed_of_sound_sea_level).max(0.0);
    // *** CORRECTION: Use fixed factor as config field doesn't exist, ensure f64 types ***
    let ram_factor = (1.0_f64 - 0.1_f64 * mach).max(0.0_f64); // Ensure non-negative and use f64 literals

    // Calculate available thrust range at current conditions
    let max_thrust_available = (config.max_thrust * rho_factor * ram_factor).max(0.0);
    let min_thrust_available = (config.min_thrust * rho_factor).max(0.0); // Assume min thrust also affected by density
    let effective_max_thrust = max_thrust_available.max(min_thrust_available); // Ensure max >= min

    // Calculate actual thrust based on the engine's current internal state (thrust_fraction)
    let thrust = min_thrust_available
        + (effective_max_thrust - min_thrust_available) * state.thrust_fraction;

    // Calculate fuel flow based on thrust and TSFC
    let fuel_flow = if state.running && state.thrust_fraction > 1e-6 {
        // Check if effectively running
        let current_tsfc = config.tsfc; // Assuming constant TSFC from config
                                        // *** CORRECTION: Use fixed factor as config field doesn't exist ***
        let inefficiency_factor = 1.0 + 0.2 * state.thrust_fraction; // Simplified inefficiency
                                                                     // Fuel flow = Thrust * TSFC * Inefficiency
        thrust * current_tsfc * inefficiency_factor
    } else {
        0.0 // No fuel flow if engine is off or at zero thrust fraction
    };

    (thrust.max(0.0), fuel_flow.max(0.0)) // Ensure non-negative outputs
}

// --- New Pure Calculation Function ---

/// Contains the outputs calculated for a single engine in a given state.
#[derive(Debug, Clone)] // Added Clone derive
pub struct EngineOutputs {
    /// The calculated Force component (vector, application point) for this engine.
    pub force_component: Force,
    /// The calculated fuel flow rate (scalar) for this engine.
    pub fuel_flow: f64,
    // Add moment_component: Option<Moment> if engines produce significant moments (e.g., torque)
}

/// Calculates the force component and fuel flow for a single engine based on its current state.
/// Pure function: Takes data, returns calculated outputs.
/// Assumes `engine_state` has already been updated for the current frame by `update_powerplant_state`.
pub fn calculate_engine_outputs(
    config: &PowerplantConfig,
    state: &PowerplantState, // Should be the state *after* update_powerplant_state
    air_density: f64,
    airspeed: f64,
) -> EngineOutputs {
    // 1. Calculate Thrust Scalar and Fuel Flow
    let (thrust_scalar, fuel_flow) =
        calculate_thrust_and_fuel_flow(config, state, air_density, airspeed);

    // 2. Determine Thrust Vector Direction in Body Frame
    let thrust_direction_unit: UnitQuaternion<f64> = UnitQuaternion::from_euler_angles(
        config.orientation.x, // Roll offset
        config.orientation.y, // Pitch offset
        config.orientation.z, // Yaw offset
    );
    // Standard engine thrust is along the positive X axis *relative to the engine's orientation*
    let base_thrust_vector = Vector3::x(); // Vector [1, 0, 0]
    let thrust_direction_body = thrust_direction_unit * base_thrust_vector; // Rotate base vector

    // 3. Calculate Force Vector
    // *** CORRECTION: Multiply the direction vector (not the Unit struct) by the scalar ***
    let force_vector_body = thrust_direction_body * thrust_scalar; // nalgebra::Vector * scalar

    // 4. Create the Force Component struct
    let force_component = Force {
        vector: force_vector_body,
        point: Some(config.position), // Apply force at the engine's position relative to CG
        frame: ReferenceFrame::Body,
        category: ForceCategory::Propulsive,
    };

    // 5. Return results
    EngineOutputs {
        force_component,
        fuel_flow,
        // moment_component: None, // Add moment calculation if needed
    }
}

// --- Updated Bevy System (Wrapper) ---

/// Bevy system for calculating propulsion forces and updating engine states.
/// Calls pure calculation functions and updates Bevy components.
pub fn propulsion_system(
    mut query: Query<(
        &AircraftControlSurfaces,
        &mut PropulsionState,  // Mut needed to update engine states
        &mut PhysicsComponent, // Mut needed to add forces
        &AirData,              // Read air data
        &FullAircraftConfig,   // Read propulsion config
    )>,
    physics_config: Res<PhysicsConfig>, // Get dt
) {
    let dt = physics_config.timestep;

    for (controls, mut propulsion_state, mut physics, air_data, aircraft_config) in query.iter_mut()
    {
        // --- 1. Clear Previous Frame's Forces ---
        physics
            .forces
            .retain(|force| force.category != ForceCategory::Propulsive);
        physics
            .moments
            .retain(|moment| moment.category != ForceCategory::Propulsive);

        // --- 2. Update Overall State (e.g., power lever for all engines) ---
        // This assumes all engines respond to the same lever. If independent, update inside loop.
        propulsion_state.set_power_lever(controls.power_lever);

        // --- 3. Iterate Engines, Update State, Calculate Outputs ---
        // Collect updates to avoid mutable borrow conflicts within the loop
        let mut final_updates: Vec<(usize, PowerplantState, Force)> =
            Vec::with_capacity(aircraft_config.propulsion.engines.len());

        for (index, (engine_config, current_engine_state)) in aircraft_config
            .propulsion
            .engines
            .iter()
            .zip(propulsion_state.engine_states.iter()) // Borrow immutably here
            .enumerate()
        {
            // --- a. Update Engine's Internal State ---
            let mut next_engine_state = current_engine_state.clone(); // Clone to modify

            // Pass the power lever from the *overall* state into the individual engine state
            // before updating dynamics (in case set_power_lever didn't update individuals).
            next_engine_state.power_lever = controls.power_lever;

            update_powerplant_state(&mut next_engine_state, engine_config, dt);

            // --- b. Calculate Outputs using Pure Function ---
            // Pass the *updated* state to the calculation function
            let outputs: EngineOutputs = calculate_engine_outputs(
                engine_config,
                &next_engine_state, // Use the state reflecting this frame's dynamics
                air_data.density,
                air_data.true_airspeed,
            );

            // --- c. Store Final State and Force ---
            // Update fuel flow on the state *after* calculation for tracking
            next_engine_state.fuel_flow = outputs.fuel_flow;
            // Store index, the final state for this frame, and the calculated force
            final_updates.push((index, next_engine_state, outputs.force_component));
            // Add moment here if calculate_engine_outputs returns one
        }

        // --- 4. Apply Updates to Bevy Components ---
        for (index, final_state, force_component) in final_updates {
            // Update the engine state in the main PropulsionState component
            if let Some(state_mut) = propulsion_state.engine_states.get_mut(index) {
                *state_mut = final_state;
            }
            // Add the calculated force to the PhysicsComponent
            if force_component.vector.norm_squared() > 1e-9 {
                // Threshold check
                physics.add_force(force_component);
            }
            // Add moment if applicable
            // if let Some(moment_component) = moment_component {
            //    if moment_component.vector.norm_squared() > 1e-9 {
            //       physics.add_moment(moment_component);
            //    }
            // }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use approx::assert_relative_eq;

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
    fn test_engine_spool_down() {
        let config = PowerplantConfig::default();
        let mut state = PowerplantState::default();

        // Start with engine at full thrust
        state.power_lever = 1.0;
        state.thrust_fraction = 1.0;
        state.running = true;

        // Set throttle to idle
        state.power_lever = 0.0;

        // Update for one second
        update_powerplant_state(&mut state, &config, 1.0);

        // Check that thrust is decreasing but not immediately at zero
        assert!(state.thrust_fraction < 1.0);
        assert!(state.thrust_fraction > 0.0);
        // Engine should be turned off at idle
        assert!(!state.running);
    }

    #[test]
    fn test_engine_on_off_state() {
        let config = PowerplantConfig::default();
        let mut state = PowerplantState::default();

        // Engine initially off, then set throttle above zero
        assert!(!state.running);
        state.power_lever = 0.5;

        // Update state - should turn on
        update_powerplant_state(&mut state, &config, 0.1);
        assert!(state.running);

        // Set throttle to zero
        state.power_lever = 0.0;

        // Update state - should turn off
        update_powerplant_state(&mut state, &config, 0.1);
        assert!(!state.running);
    }

    #[test]
    fn test_propulsion_state_methods() {
        // Test multi-engine propulsion state
        let mut propulsion = PropulsionState::new(2);

        // Initially engines should be off with zero throttle
        assert_eq!(propulsion.engine_states.len(), 2);
        assert_eq!(propulsion.engine_states[0].power_lever, 0.0);
        assert_eq!(propulsion.engine_states[1].power_lever, 0.0);
        assert!(!propulsion.engine_states[0].running);
        assert!(!propulsion.engine_states[1].running);

        // Test setting power lever for all engines
        propulsion.set_power_lever(0.8);
        assert_eq!(propulsion.engine_states[0].power_lever, 0.8);
        assert_eq!(propulsion.engine_states[1].power_lever, 0.8);

        // Test setting individual engine power lever
        propulsion.set_engine_power_lever(0, 0.5);
        propulsion.set_engine_power_lever(1, 0.6);
        assert_eq!(propulsion.engine_states[0].power_lever, 0.5);
        assert_eq!(propulsion.engine_states[1].power_lever, 0.6);

        // Test turning engines on
        propulsion.turn_engines_on();
        assert!(propulsion.engine_states[0].running);
        assert!(propulsion.engine_states[1].running);
    }

    #[test]
    fn test_thrust_calculation() {
        // let config = PowerplantConfig::default();
        let mut state = PowerplantState::default();
        state.thrust_fraction = 1.0;
        state.running = true;

        // Test at sea level, zero airspeed

        // Test at altitude (lower density)

        // Test with airspeed

        // Test engine off (zero fuel flow)
    }
}
