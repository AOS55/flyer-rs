use bevy::prelude::*;
use nalgebra::Vector3;

use crate::{
    components::{
        AircraftControlSurfaces, FullAircraftConfig, FullAircraftState, PhysicsComponent,
        SpatialComponent,
    },
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, physics_integrator_system,
    },
};

/// A virtual physics simulation that can run independently of the main simulation time
pub struct VirtualPhysics {
    world: World,
    _physics_config: PhysicsConfig,
    fixed_timestep: f64,
}

impl VirtualPhysics {
    pub fn new(physics_config: PhysicsConfig, fixed_timestep: f64) -> Self {
        Self {
            world: World::new(),
            _physics_config: physics_config,
            fixed_timestep,
        }
    }

    /// Create a virtual aircraft entity with given state and config
    pub fn spawn_aircraft(
        &mut self,
        state: &FullAircraftState,
        config: &FullAircraftConfig,
    ) -> Entity {
        let mut spatial = SpatialComponent::default();
        spatial.position = state.spatial.position;
        spatial.velocity = state.spatial.velocity;
        spatial.attitude = state.spatial.attitude;
        spatial.angular_velocity = state.spatial.angular_velocity;

        let physics = PhysicsComponent::new(config.mass.mass, config.mass.inertia);

        self.world
            .spawn((spatial, physics, state.clone(), config.clone()))
            .id()
    }

    /// Run the physics simulation for a specified number of steps
    pub fn run_steps(&mut self, _entity: Entity, steps: usize) {
        let _dt = self.fixed_timestep;

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
    pub fn get_state(&self, entity: Entity) -> FullAircraftState {
        self.world
            .get::<FullAircraftState>(entity)
            .expect("Entity should have FullAircraftState")
            .clone()
    }

    /// Set the state of the virtual aircraft
    pub fn set_state(&mut self, entity: Entity, state: &FullAircraftState) {
        if let Some(mut entity_state) = self.world.get_mut::<FullAircraftState>(entity) {
            *entity_state = state.clone();
        }

        if let Some(mut spatial) = self.world.get_mut::<SpatialComponent>(entity) {
            spatial.position = state.spatial.position;
            spatial.velocity = state.spatial.velocity;
            spatial.attitude = state.spatial.attitude;
            spatial.angular_velocity = state.spatial.angular_velocity;
        }
    }

    /// Set control inputs for the virtual aircraft
    pub fn set_controls(&mut self, entity: Entity, controls: &AircraftControlSurfaces) {
        if let Some(mut state) = self.world.get_mut::<FullAircraftState>(entity) {
            state.control_surfaces = *controls;
        }
    }
}
