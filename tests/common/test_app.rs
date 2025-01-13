use bevy::prelude::*;
use flyer::{
    components::{
        AircraftConfig, DubinsAircraftConfig, FullAircraftConfig, TrimBounds, TrimSolverConfig,
    },
    plugins::*,
    resources::{EnvironmentConfig, PhysicsConfig, TerrainConfig, UpdateMode},
    systems::{
        aero_force_system, air_data_system, dubins_aircraft_system, force_calculator_system,
        handle_trim_requests, physics_integrator_system, propulsion_system, trim_aircraft_system,
    },
};

// Builder for creating a test application with customizable configuration
pub struct TestAppBuilder {
    aircraft_configs: Vec<AircraftConfig>,
    physics_config: Option<PhysicsConfig>,
    environment_config: Option<EnvironmentConfig>,
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
            environment_config: None,
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

    pub fn with_environment(mut self, config: EnvironmentConfig) -> Self {
        self.environment_config = Some(config);
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

    pub fn build(self) -> TestApp {
        let mut app = App::new();

        // Add required plugins
        app.add_plugins(MinimalPlugins)
            .add_plugins(TransformationPlugin::new(1.0))
            .add_plugins(PhysicsPlugin::with_config(
                self.physics_config.unwrap_or_default(),
            ))
            .add_plugins(EnvironmentPlugin::with_config(
                self.environment_config.unwrap_or_default(),
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

        // app.add_systems(Update, || println!("Running Update"));

        // Add Dubins systems
        app.add_systems(Update, dubins_aircraft_system);

        // app.add_systems(Update, || println!("Update schedule is running"));

        // Add Full systems
        app.add_systems(
            Update,
            (
                (|| println!("Before air_data_system")),
                air_data_system,
                (|| println!("After air_data_system")),
                aero_force_system,
                (|| println!("After aero_force_system")),
                propulsion_system,
                (|| println!("After propulsion_system")),
                force_calculator_system,
                (|| println!("After force_calculator_system")),
                physics_integrator_system,
                (|| println!("After physics_integrator_system")),
            )
                .chain(),
        );

        // Add Trim systems (should be plugin)
        // app.add_systems(Update, (handle_trim_requests, trim_aircraft_system).chain());

        // app.insert_resource(TrimSolverConfig {
        //     max_iterations: 1000,
        //     cost_tolerance: 1e-6,
        //     state_tolerance: 1e-8,
        //     use_gradient_refinement: true,
        //     bounds: TrimBounds::default(),
        // });

        // Run an initial update to initialize everything
        app.update();

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
        println!("Running {} steps", steps);
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
