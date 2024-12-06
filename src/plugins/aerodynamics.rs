use bevy::prelude::*;

use crate::components::{
    AeroCoefficients, AerodynamicsComponent, AirData, AircraftGeometry, ControlSurfaces,
};
use crate::config::aerodynamics::AerodynamicsConfig;
use crate::systems::aerodynamics::{aero_force_system, air_data_system};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum AerodynamicsSet {
    AirData,
    ForceCalculation,
}

pub struct AerodynamicsPlugin;

impl Plugin for AerodynamicsPlugin {
    fn build(&self, app: &mut App) {
        // Register components and resources
        app.register_type::<AerodynamicsComponent>()
            .register_type::<AircraftGeometry>()
            .register_type::<AirData>()
            .register_type::<AeroCoefficients>()
            .register_type::<ControlSurfaces>()
            .init_resource::<AerodynamicsConfig>();

        // Configure system sets
        app.configure_sets(
            FixedUpdate,
            (AerodynamicsSet::AirData, AerodynamicsSet::ForceCalculation)
                .chain()
                .after(PhysicsSet::Integration),
        );

        // Add systems
        app.add_systems(
            FixedUpdate,
            (
                air_data_system.in_set(AerodynamicsSet::AirData),
                aero_force_system.in_set(AerodynamicsSet::ForceCalculation),
            ),
        );
    }
}
