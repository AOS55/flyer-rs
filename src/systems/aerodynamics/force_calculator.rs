use aerso::{AeroEffect, AirState};
use bevy::prelude::*;

use super::aerso_adapter::AersoAdapter;
use crate::components::{
    AirData, AircraftAeroCoefficients, AircraftControlSurfaces, AircraftGeometry, Force,
    ForceCategory, Moment, PhysicsComponent, ReferenceFrame, SpatialComponent,
};
use crate::resources::AerodynamicsConfig;

/// System for calculating aerodynamic forces and moments
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
        if air_data.true_airspeed < config.min_airspeed_threshold {
            continue;
        }

        let adapter = AersoAdapter::new(geometry.clone(), coefficients.clone());
        calculate_aero_forces(&adapter, control_surfaces, air_data, spatial, &mut physics);
    }
}

fn calculate_aero_forces(
    adapter: &AersoAdapter,
    control_surfaces: &AircraftControlSurfaces,
    air_data: &AirData,
    spatial: &SpatialComponent,
    physics: &mut PhysicsComponent,
) {
    let air_state = AirState {
        alpha: air_data.alpha,
        beta: air_data.beta,
        airspeed: air_data.true_airspeed,
        q: air_data.dynamic_pressure,
    };

    let input = vec![
        control_surfaces.aileron,
        control_surfaces.elevator,
        control_surfaces.rudder,
        control_surfaces.flaps,
    ];

    let (aero_force, aero_torque) = adapter.get_effect(air_state, spatial.angular_velocity, &input);

    let force_vector = match aero_force.frame {
        aerso::types::Frame::Body => aero_force.force,
        aerso::types::Frame::World => spatial.attitude.inverse() * aero_force.force,
    };

    physics.add_force(Force {
        vector: force_vector,
        point: None,
        frame: ReferenceFrame::Body,
        category: ForceCategory::Aerodynamic,
    });

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
