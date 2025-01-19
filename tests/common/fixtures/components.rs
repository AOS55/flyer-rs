use flyer::components::{
    AircraftConfig, AircraftControlSurfaces, DubinsAircraftConfig, FixedStartConfig,
    FullAircraftConfig, RandomStartConfig, SpatialComponent, StartConfig,
};
use nalgebra::{UnitQuaternion, Vector3};
use std::{f64::consts::PI, sync::LazyLock};

pub mod fixtures {
    use super::*;

    /// Standard test aircraft configuration
    pub static TEST_AIRCRAFT_CONFIG: LazyLock<AircraftConfig> = LazyLock::new(|| {
        AircraftConfig::Dubins(DubinsAircraftConfig {
            name: String::new(), // Will be set during tests
            max_speed: 100.0,
            min_speed: 20.0,
            acceleration: 5.0,
            max_bank_angle: PI / 4.0,
            max_turn_rate: 0.5,
            max_climb_rate: 10.0,
            max_descent_rate: 10.0,
            start_config: StartConfig::default(),
            task_config: Default::default(),
        })
    });
}

/// Creates a spatial component in straight and level flight
pub fn straight_level_spatial() -> SpatialComponent {
    SpatialComponent {
        position: Vector3::new(0.0, 0.0, -1000.0),
        velocity: Vector3::new(50.0, 0.0, 0.0),
        attitude: UnitQuaternion::identity(),
        angular_velocity: Vector3::zeros(),
    }
}

/// Creates a spatial component in climbing flight
pub fn climbing_spatial() -> SpatialComponent {
    SpatialComponent {
        position: Vector3::new(0.0, 0.0, -1000.0),
        velocity: Vector3::new(50.0, 0.0, -5.0),
        attitude: UnitQuaternion::from_euler_angles(0.0, 0.1, 0.0),
        angular_velocity: Vector3::zeros(),
    }
}

/// Creates a spatial component in turning flight
pub fn turning_spatial() -> SpatialComponent {
    SpatialComponent {
        position: Vector3::new(0.0, 0.0, -1000.0),
        velocity: Vector3::new(50.0, 0.0, 0.0),
        attitude: UnitQuaternion::from_euler_angles(PI / 6.0, 0.0, 0.0),
        angular_velocity: Vector3::new(0.0, 0.0, 0.2),
    }
}

/// Creates neutral control surface positions
pub fn neutral_controls() -> AircraftControlSurfaces {
    AircraftControlSurfaces {
        elevator: 0.0,
        aileron: 0.0,
        rudder: 0.0,
        power_lever: 0.0,
    }
}

/// Test aircraft configurations for different scenarios
pub mod aircraft_configs {
    use super::*;

    pub fn basic_dubins() -> DubinsAircraftConfig {
        DubinsAircraftConfig {
            name: "test_basic_dubins".to_string(),
            max_speed: 100.0,
            min_speed: 20.0,
            acceleration: 5.0,
            max_bank_angle: PI / 4.0,
            max_turn_rate: 0.5,
            max_climb_rate: 10.0,
            max_descent_rate: 10.0,
            start_config: StartConfig::Fixed(FixedStartConfig::default()),
            task_config: Default::default(),
        }
    }

    pub fn advanced_dubins() -> DubinsAircraftConfig {
        DubinsAircraftConfig {
            name: "test_advanced_dubins".to_string(),
            max_speed: 150.0,
            min_speed: 30.0,
            acceleration: 10.0,
            max_bank_angle: PI / 3.0,
            max_turn_rate: 1.0,
            max_climb_rate: 20.0,
            max_descent_rate: 15.0,
            start_config: StartConfig::Random(RandomStartConfig::default()),
            task_config: Default::default(),
        }
    }

    pub fn basic_full() -> FullAircraftConfig {
        FullAircraftConfig::twin_otter()
    }

    pub fn high_performance() -> FullAircraftConfig {
        FullAircraftConfig::f4_phantom()
    }
}
