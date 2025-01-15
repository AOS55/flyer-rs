use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    components::{
        AirData, AircraftControlSurfaces, FullAircraftConfig, PhysicsComponent, PropulsionState,
        SpatialComponent,
    },
    resources::{AerodynamicsConfig, EnvironmentConfig, EnvironmentModel, PhysicsConfig},
    systems::{
        aero_force_system, air_data_system, force_calculator_system, physics_integrator_system,
    },
};

/// A virtual physics simulation that can run independently of the main simulation time
#[derive(Debug)]
pub struct VirtualPhysics {
    world: World,
}

impl VirtualPhysics {
    pub fn new(physics_config: &PhysicsConfig) -> Self {
        let mut world = World::new();

        world.insert_resource(physics_config.clone());

        let env_config = EnvironmentConfig::default(); // Assume nill wind for now
        world.insert_resource(EnvironmentModel::new(&env_config));
        world.insert_resource(AerodynamicsConfig {
            min_airspeed_threshold: 0.0,
        });

        Self { world }
    }

    /// Create a virtual aircraft entity with given state and config
    pub fn spawn_aircraft(
        &mut self,
        spatial: &SpatialComponent,
        propulsion: &PropulsionState,
        config: &FullAircraftConfig,
    ) -> Entity {
        let air_data = AirData::default();
        let control_surfaces = AircraftControlSurfaces::default();
        let spatial = spatial.clone();
        let physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);
        let propulsion = propulsion.clone();
        self.world
            .spawn((
                air_data,
                control_surfaces,
                spatial,
                physics,
                propulsion,
                config.clone(),
            ))
            .id()
    }

    /// Run the physics simulation for a specified number of steps
    pub fn run_steps(&mut self, _entity: Entity, steps: usize) {
        for _ in 0..steps {
            // Create a fake time resource for the fixed timestep
            // TODO: Ensure the update occurs for dt on each but can go faster than dt
            // let time = Time::new_fixed(Duration::from_secs_f64(dt));
            // self.world.insert_resource(time);

            // Run the physics systems in sequence
            let mut air_data_schedule = Schedule::default();
            air_data_schedule.add_systems(air_data_system);
            air_data_schedule.run(&mut self.world);

            let mut aero_schedule = Schedule::default();
            aero_schedule.add_systems(aero_force_system);
            aero_schedule.run(&mut self.world);

            let mut force_schedule = Schedule::default();
            force_schedule.add_systems(force_calculator_system);
            force_schedule.run(&mut self.world);

            let mut integrator_schedule = Schedule::default();
            integrator_schedule.add_systems(physics_integrator_system);
            integrator_schedule.run(&mut self.world);
        }
    }

    /// Calculate forces and moments at current state without integrating
    pub fn calculate_forces(&mut self, entity: Entity) -> (Vector3<f64>, Vector3<f64>) {
        // Run force calculation systems without integration
        let mut air_data_schedule = Schedule::default();
        air_data_schedule.add_systems(air_data_system);
        air_data_schedule.run(&mut self.world);

        let mut aero_schedule = Schedule::default();
        aero_schedule.add_systems(aero_force_system);
        aero_schedule.run(&mut self.world);

        let mut force_schedule = Schedule::default();
        force_schedule.add_systems(force_calculator_system);
        force_schedule.run(&mut self.world);

        // Get the resulting forces and moments
        let physics = self
            .world
            .get::<PhysicsComponent>(entity)
            .expect("Entity should have PhysicsComponent");

        (physics.net_force, physics.net_moment)
    }

    /// Get the current state of the virtual aircraft
    pub fn get_state(&self, entity: Entity) -> (SpatialComponent, AircraftControlSurfaces) {
        let spatial = self
            .world
            .get::<SpatialComponent>(entity)
            .expect("Entity should have FullAircraftState")
            .clone();

        let control_surfaces = self
            .world
            .get::<AircraftControlSurfaces>(entity)
            .expect("Entity should have AircraftControlSurfaces")
            .clone();

        (spatial, control_surfaces)
    }

    /// Set the state of the virtual aircraft
    pub fn set_state(
        &mut self,
        entity: Entity,
        velocity: &Vector3<f64>,
        attitude: &UnitQuaternion<f64>,
    ) {
        if let Some(mut entity_spatial) = self.world.get_mut::<SpatialComponent>(entity) {
            entity_spatial.velocity = velocity.clone();
            entity_spatial.attitude = attitude.clone();
        }
    }

    /// Set control inputs for the virtual aircraft
    pub fn set_controls(&mut self, entity: Entity, controls: &AircraftControlSurfaces) {
        if let Some(mut control_surfaces) = self.world.get_mut::<AircraftControlSurfaces>(entity) {
            *control_surfaces = controls.clone();
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_physics_initialization() {
        // Test aircraft entity creation
        // Verify initial state setting
        // Check component setup
    }

    #[test]
    fn test_physics_step() {
        // Test single physics step
        // Verify force calculations
        // Check state integration
    }

    #[test]
    fn test_force_calculation() {
        // Test force/moment calculation
        // Verify coordinate transformations
        // Check boundary conditions
    }
}
