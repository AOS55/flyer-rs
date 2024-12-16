use bevy::prelude::*;
use nalgebra::{Matrix3, Vector3};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::components::{PhysicsComponent, RandomStartPosConfig, SpatialComponent};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AircraftState {
    pub air_data: AirData,
    pub control_surfaces: AircraftControlSurfaces,
    pub spatial: SpatialComponent,
    pub physics: PhysicsComponent,
}

impl Default for AircraftState {
    fn default() -> Self {
        Self {
            air_data: AirData::default(),
            control_surfaces: AircraftControlSurfaces::default(),
            spatial: SpatialComponent::default(),
            physics: PhysicsComponent::new(
                4874.8,
                Matrix3::from_columns(&[
                    Vector3::new(28366.4, 0.0, -1384.3),
                    Vector3::new(0.0, 32852.8, 0.0),
                    Vector3::new(-1384.3, 0.0, 52097.3),
                ]),
            ),
        }
    }
}

// TODO: Implement a method to make a new aircraft from a state
// impl AircraftState {
//     pub fn new() -> Self {}
// }

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AircraftControlSurfaces {
    pub elevator: f64,
    pub aileron: f64,
    pub rudder: f64,
    pub flaps: f64,
}

impl Default for AircraftControlSurfaces {
    fn default() -> Self {
        Self {
            elevator: 0.0,
            aileron: 0.0,
            rudder: 0.0,
            flaps: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AirData {
    pub true_airspeed: f64,
    pub alpha: f64,
    pub beta: f64,
    pub dynamic_pressure: f64,
    pub density: f64,
    pub relative_velocity: Vector3<f64>,
    pub wind_velocity: Vector3<f64>,
}

impl Default for AirData {
    fn default() -> Self {
        Self {
            true_airspeed: 0.0,
            alpha: 0.0,
            beta: 0.0,
            dynamic_pressure: 0.0,
            density: 0.0,
            relative_velocity: Vector3::zeros(),
            wind_velocity: Vector3::zeros(),
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DubinsAircraftState {
    pub spatial: SpatialComponent,
    pub controls: DubinsAircraftControls,
}

impl Default for DubinsAircraftState {
    fn default() -> Self {
        Self {
            spatial: SpatialComponent::default(),
            controls: DubinsAircraftControls::default(),
        }
    }
}

impl DubinsAircraftState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start the vehicle at an exact position in NED coordinates
    pub fn precise_position(position: Vector3<f64>) -> Self {
        let spatial = SpatialComponent::at_position(position);
        let controls = DubinsAircraftControls::default();

        Self { spatial, controls }
    }

    /// Start the vehicle at a random position on a hemisphere
    pub fn random_position(config: Option<RandomStartPosConfig>) -> Self {
        let mut config = config.unwrap_or_default();

        let loc_min_altitude = config.min_altitude;
        let loc_max_altitude = config.max_altitude;
        let loc_origin = config.origin.clone();

        let (loc_min_altitude, loc_max_altitude) = if loc_min_altitude < loc_max_altitude {
            (loc_min_altitude, loc_max_altitude)
        } else {
            warn!(
                "Invalid altitude range: min_altitude ({}) >= max_altitude ({}). Swapping values.",
                loc_min_altitude, loc_max_altitude
            );
            (loc_max_altitude, loc_min_altitude)
        };

        let u1 = config.rng.gen::<f64>();
        let u2 = config.rng.gen::<f64>();
        let radius = config.variance * (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;

        let x = loc_origin.x + radius * theta.cos();
        let y = loc_origin.y + radius * theta.sin();
        let z = config.rng.gen_range(loc_min_altitude..loc_max_altitude);

        // info!("x: {}, y: {}, z: {}", x, y, z);

        let position = loc_origin.push(0.0) + Vector3::new(x, y, z);
        let spatial = SpatialComponent::at_position(position);

        Self {
            spatial,
            controls: DubinsAircraftControls::default(),
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DubinsAircraftControls {
    pub acceleration: f64,
    pub bank_angle: f64,
    pub vertical_speed: f64,
}

impl Default for DubinsAircraftControls {
    fn default() -> Self {
        Self {
            acceleration: 0.0,
            bank_angle: 0.0,
            vertical_speed: 0.0,
        }
    }
}
