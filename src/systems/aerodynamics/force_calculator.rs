use aerso::{AeroEffect, AirState};
use bevy::prelude::*;

use super::aerso_adapter::AersoAdapter;
use crate::components::{
    AirData, AircraftControlSurfaces, Force, ForceCategory, FullAircraftConfig, Moment,
    PhysicsComponent, ReferenceFrame, SpatialComponent,
};
use crate::resources::AerodynamicsConfig;

/// System for calculating aerodynamic forces and moments acting on aircraft.
///
/// This system computes the aerodynamic forces and moments based on the aircraft's geometry,
/// aerodynamic coefficients, control surface inputs, air data, and spatial properties. The
/// results are added to the aircraft's physics component.
pub fn aero_force_system(
    mut aircraft: Query<(
        &AircraftControlSurfaces,
        &AirData,
        &SpatialComponent,
        &mut PhysicsComponent,
        &FullAircraftConfig,
    )>,
    aero_config: Res<AerodynamicsConfig>,
) {
    // println!("Running Aero Force System!");
    for (controls, air_data, spatial, mut physics, config) in aircraft.iter_mut() {
        // Early return if airspeed is below threshold
        if air_data.true_airspeed < aero_config.min_airspeed_threshold {
            continue;
        }

        // Create adapter outside of the calculation
        let adapter = AersoAdapter::new(config.geometry.clone(), config.aero_coef.clone());

        // Collect all necessary data before calculation
        let aero_forces = prepare_aero_forces(&adapter, &controls, &air_data, &spatial);
        // println!("Forces Config: {:?}", config);

        // Clear existing aerodynamic forces and moments before adding new ones
        physics
            .forces
            .retain(|force| force.category != ForceCategory::Aerodynamic);
        physics
            .moments
            .retain(|moment| moment.category != ForceCategory::Aerodynamic);

        // Apply the calculated forces and moments
        apply_aero_forces(&mut physics, aero_forces);
    }
}

/// Represents the calculated aerodynamic forces and moments
struct AeroForces {
    force: Force,
    moment: Moment,
}

/// Prepares aerodynamic forces and moments without mutating any state
fn prepare_aero_forces(
    adapter: &AersoAdapter,
    control_surfaces: &AircraftControlSurfaces,
    air_data: &AirData,
    spatial: &SpatialComponent,
) -> AeroForces {
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
        control_surfaces.power_lever,
    ];

    let (aero_force, aero_torque) = adapter.get_effect(air_state, spatial.angular_velocity, &input);

    let force_vector = match aero_force.frame {
        aerso::types::Frame::Body => aero_force.force,
        aerso::types::Frame::World => spatial.attitude.inverse() * aero_force.force,
    };

    AeroForces {
        force: Force {
            vector: force_vector,
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        },
        moment: Moment {
            vector: aero_torque.torque,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        },
    }
}

/// Applies the calculated forces and moments to the physics component
fn apply_aero_forces(physics: &mut PhysicsComponent, aero_forces: AeroForces) {
    physics.add_force(aero_forces.force);
    physics.add_moment(aero_forces.moment);
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
