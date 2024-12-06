use crate::systems::physics::{
    force_calculator_system, physics_integrator_system, ForceCalculatorConfig, IntegratorConfig,
};
use bevy::prelude::*;

/// Physics simulation stages
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PhysicsSet {
    ForceCalculation,
    Integration,
}

pub struct PhysicsPlugin {
    pub timestep: f64,
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self {
            timestep: 1.0 / 120.0, // 120 Hz default physics rate
        }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Register components for reflection
        app.register_type::<crate::components::PhysicsComponent>()
            .register_type::<crate::components::SpatialComponent>();

        // Add resources
        app.init_resource::<ForceCalculatorConfig>()
            .init_resource::<IntegratorConfig>();

        // Configure fixed timestep
        app.insert_resource(Time::<Fixed>::from_seconds_f64(self.timestep));

        // Add systems in the correct order
        app.configure_sets(
            FixedUpdate,
            (PhysicsSet::ForceCalculation, PhysicsSet::Integration).chain(),
        );

        app.add_systems(
            FixedUpdate,
            (
                force_calculator_system.in_set(PhysicsSet::ForceCalculation),
                physics_integrator_system.in_set(PhysicsSet::Integration),
            ),
        );
    }
}
