use crate::common::TestApp;
use bevy::prelude::*;
use flyer::{
    components::{
        AircraftAeroCoefficients, AircraftConfig, AircraftGeometry, AircraftType,
        DubinsAircraftConfig, FixedStartConfig, FullAircraftConfig, MassModel, PropulsionConfig,
        RandomStartConfig, SpatialComponent, StartConfig,
    },
    resources::{PhysicsConfig, TerrainConfig},
};
use nalgebra::{UnitQuaternion, Vector3};
use std::f64::consts::PI;

/// Creates a simple test spatial component
pub fn create_test_spatial() -> SpatialComponent {
    SpatialComponent {
        position: Vector3::new(0.0, 0.0, -1000.0),
        velocity: Vector3::new(50.0, 0.0, 0.0),
        attitude: UnitQuaternion::identity(),
        angular_velocity: Vector3::zeros(),
    }
}

/// Creates a test physics configuration
pub fn create_test_physics_config() -> PhysicsConfig {
    PhysicsConfig {
        max_velocity: 200.0,
        max_angular_velocity: 10.0,
        timestep: 1.0 / 120.0,
        gravity: Vector3::new(0.0, 0.0, 9.81),
    }
}

/// Creates a basic Dubins aircraft configuration for testing
pub fn create_test_dubins_config() -> DubinsAircraftConfig {
    DubinsAircraftConfig {
        name: "test_dubins".to_string(),
        max_speed: 100.0,
        min_speed: 20.0,
        acceleration: 5.0,
        max_bank_angle: PI / 4.0,
        max_turn_rate: 0.5,
        max_climb_rate: 10.0,
        max_descent_rate: 10.0,
        start_config: StartConfig::Fixed(FixedStartConfig::default()),
    }
}

/// Creates a basic full aircraft configuration for testing
pub fn create_test_full_config() -> FullAircraftConfig {
    FullAircraftConfig {
        name: "test_full".to_string(),
        ac_type: AircraftType::TwinOtter,
        mass: MassModel::twin_otter(),
        geometry: AircraftGeometry::twin_otter(),
        aero_coef: AircraftAeroCoefficients::twin_otter(),
        propulsion: PropulsionConfig::twin_otter(),
        start_config: StartConfig::Fixed(FixedStartConfig {
            position: Vector3::new(0.0, 0.0, -600.0),
            speed: 200.0,
            heading: 0.0,
        }),
    }
}

/// Creates a test random start configuration
pub fn create_test_random_start() -> RandomStartConfig {
    RandomStartConfig::default()
}

/// Creates a simple test terrain configuration
pub fn create_test_terrain_config() -> TerrainConfig {
    TerrainConfig::default()
}

/// Waits for a specific condition to be met within a maximum number of steps
pub fn wait_for_condition<F>(test_app: &mut TestApp, condition: F, max_steps: usize) -> bool
where
    F: Fn(&mut App) -> bool,
{
    for _ in 0..max_steps {
        if condition(&mut test_app.app) {
            return true;
        }
        test_app.run_frame();
    }
    false
}

/// Creates an aircraft configuration for different test scenarios
pub fn create_aircraft_config(scenario: TestScenario) -> AircraftConfig {
    match scenario {
        TestScenario::BasicFlight => AircraftConfig::Dubins(create_test_dubins_config()),
        TestScenario::AdvancedFlight => {
            let mut config = create_test_dubins_config();
            config.max_speed = 150.0;
            config.max_bank_angle = PI / 3.0;
            AircraftConfig::Dubins(config)
        }
        TestScenario::FullPhysics => AircraftConfig::Full(create_test_full_config()),
    }
}

/// Common test scenarios
#[derive(Debug, Clone, Copy)]
pub enum TestScenario {
    BasicFlight,
    AdvancedFlight,
    FullPhysics,
}

/// Helper to run a simulation for a specific duration
pub fn simulate_duration(test_app: &mut TestApp, duration: f64, timestep: f64) {
    let steps = (duration / timestep).ceil() as usize;
    test_app.run_steps(steps);
}
