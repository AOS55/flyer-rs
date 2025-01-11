use bevy::prelude::*;
use flyer::{
    components::{AircraftConfig, DubinsAircraftConfig, FullAircraftConfig},
    plugins::*,
    resources::{PhysicsConfig, TerrainConfig, UpdateMode},
};

// Builder for creating a test application with customizable configuration
pub struct TestAppBuilder {
    aircraft_configs: Vec<AircraftConfig>,
    physics_config: Option<PhysicsConfig>,
    terrain_config: Option<TerrainConfig>,
    update_mode: UpdateMode,
    steps_per_action: usize,
    time_step: f64,
}

impl Default for TestAppBuilder {
    fn default() -> Self {
        Self {
            aircraft_configs: Vec::new(),
            physics_config: None,
            terrain_config: None,
            update_mode: UpdateMode::Gym,
            steps_per_action: 10,
            time_step: 1.0 / 120.0,
        }
    }
}

impl TestAppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dubins_aircraft(mut self, config: DubinsAircraftConfig) -> Self {
        self.aircraft_configs.push(AircraftConfig::Dubins(config));
        self
    }

    pub fn with_full_aircraft(mut self, config: FullAircraftConfig) -> Self {
        self.aircraft_configs.push(AircraftConfig::Full(config));
        self
    }

    pub fn with_physics(mut self, config: PhysicsConfig) -> Self {
        self.physics_config = Some(config);
        self
    }

    pub fn with_terrain(mut self, config: TerrainConfig) -> Self {
        self.terrain_config = Some(config);
        self
    }

    pub fn with_update_mode(mut self, mode: UpdateMode) -> Self {
        self.update_mode = mode;
        self
    }

    pub fn with_steps_per_action(mut self, steps: usize) -> Self {
        self.steps_per_action = steps;
        self
    }

    pub fn with_time_step(mut self, dt: f64) -> Self {
        self.time_step = dt;
        self
    }

    pub fn build(self) -> TestApp {
        let mut app = App::new();

        // Add required plugins
        app.add_plugins(MinimalPlugins)
            .add_plugins(TransformationPlugin::new(1.0))
            .add_plugins(PhysicsPlugin::with_config(
                self.physics_config.unwrap_or_default(),
            ))
            .add_plugins(StartupSequencePlugin);

        // Add aircraft plugins
        for config in self.aircraft_configs {
            add_aircraft_plugin(&mut app, config);
        }

        // Add terrain if configured
        if let Some(terrain_config) = self.terrain_config {
            app.add_plugins(TerrainPlugin::with_config(terrain_config));
        }

        // Configure time step
        app.insert_resource(Time::<Fixed>::from_seconds(self.time_step));

        TestApp {
            app,
            steps_per_action: self.steps_per_action,
        }
    }
}

/// Main test application wrapper
pub struct TestApp {
    pub app: App,
    pub steps_per_action: usize,
}

impl TestApp {
    pub fn run_steps(&mut self, steps: usize) {
        for _ in 0..steps {
            self.app.update();
        }
    }

    pub fn run_frame(&mut self) {
        self.app.update();
    }

    pub fn get_state<T: Resource>(&self) -> Option<&T> {
        self.app.world().get_resource::<T>()
    }

    pub fn get_state_mut<T: Resource>(&mut self) -> Option<Mut<T>> {
        self.app.world_mut().get_resource_mut::<T>()
    }

    pub fn query_single<T: Component>(&mut self) -> Option<&T> {
        let world = self.app.world_mut();
        let mut query = world.query::<&T>();
        query.get_single(world).ok()
    }

    pub fn query_single_mut<T: Component>(&mut self) -> Option<Mut<T>> {
        let world = self.app.world_mut();
        let mut query = world.query::<&mut T>();
        query.get_single_mut(world).ok()
    }

    pub fn query_all<T: Component>(&mut self) -> Vec<&T> {
        let world = self.app.world_mut();
        let mut query = world.query::<&T>();
        query.iter(world).collect()
    }
}
