use aerso::{AeroEffect, AirState};
use bevy::prelude::*;

use super::aerso_adapter::AersoAdapter;
use crate::components::{
    AirData, AircraftAeroCoefficients, AircraftControlSurfaces, AircraftGeometry, Force,
    ForceCategory, Moment, PhysicsComponent, ReferenceFrame, SpatialComponent,
};
use crate::resources::AerodynamicsConfig;

/// System for calculating aerodynamic forces and moments acting on aircraft.
///
/// This system computes the aerodynamic forces and moments based on the aircraft's geometry,
/// aerodynamic coefficients, control surface inputs, air data, and spatial properties. The
/// results are added to the aircraft's physics component.
pub fn aero_force_system(
    mut query: Query<(
        &AircraftControlSurfaces,
        &AircraftAeroCoefficients,
        &AirData,
        &SpatialComponent,
        &AircraftGeometry,
        &mut PhysicsComponent,
    )>,
    config: Res<AerodynamicsConfig>,
) {
    for (control_surfaces, coefficients, air_data, spatial, geometry, mut physics) in
        query.iter_mut()
    {
        // Skip calculations if the airspeed is below the minimum threshold
        if air_data.true_airspeed < config.min_airspeed_threshold {
            continue;
        }

        // Create an aerodynamic adapter for the current aircraft configuration
        let adapter = AersoAdapter::new(geometry.clone(), coefficients.clone());
        // Perform aerodynamic force and moment calculations
        calculate_aero_forces(&adapter, control_surfaces, air_data, spatial, &mut physics);
    }
}

/// Helper function to calculate aerodynamic forces and moments for a single entity.
///
/// # Arguments
/// * `adapter` - The aerodynamic adapter configured for the aircraft.
/// * `control_surfaces` - The state of the aircraft's control surfaces.
/// * `air_data` - The current air data for the aircraft (e.g., airspeed, alpha, beta).
/// * `spatial` - The spatial component describing the aircraft's velocity and attitude.
/// * `physics` - The physics component where forces and moments will be applied.
fn calculate_aero_forces(
    adapter: &AersoAdapter,
    control_surfaces: &AircraftControlSurfaces,
    air_data: &AirData,
    spatial: &SpatialComponent,
    physics: &mut PhysicsComponent,
) {
    // Create the air state required for aerodynamic calculations
    let air_state = AirState {
        alpha: air_data.alpha,
        beta: air_data.beta,
        airspeed: air_data.true_airspeed,
        q: air_data.dynamic_pressure,
    };

    // Prepare the input vector from control surface deflections
    let input = vec![
        control_surfaces.aileron,
        control_surfaces.elevator,
        control_surfaces.rudder,
        control_surfaces.flaps,
    ];

    // Compute aerodynamic force and torque
    let (aero_force, aero_torque) = adapter.get_effect(air_state, spatial.angular_velocity, &input);

    // Transform aerodynamic force into the body frame if necessary
    let force_vector = match aero_force.frame {
        aerso::types::Frame::Body => aero_force.force,
        aerso::types::Frame::World => spatial.attitude.inverse() * aero_force.force,
    };

    // Add aerodynamic force to the physics component
    physics.add_force(Force {
        vector: force_vector,
        point: None,
        frame: ReferenceFrame::Body,
        category: ForceCategory::Aerodynamic,
    });

    // Add aerodynamic moment to the physics component
    physics.add_moment(Moment {
        vector: aero_torque.torque,
        frame: ReferenceFrame::Body,
        category: ForceCategory::Aerodynamic,
    });
}

#[cfg(test)]
mod tests {

    #[allow(dead_code)]
    fn setup_test_app() {}

    #[allow(dead_code)]
    fn spawn_test_aircraft() {}

    #[test]
    fn test_basic_force_calculation() {}

    #[test]
    fn test_zero_airspeed_condition() {}

    #[test]
    fn test_control_surface_moments() {}

    #[test]
    fn test_attitude_effects() {}

    #[test]
    fn test_combined_effects() {}
}
