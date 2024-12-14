use bevy::prelude::*;
use nalgebra::{Matrix3, Vector2, Vector3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::components::{PhysicsComponent, SpatialComponent};

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
    pub fn random_position(
        origin: Option<Vector2<f64>>,
        variance: Option<f64>,
        min_altitude: Option<f64>,
        max_altitude: Option<f64>,
        rng: Option<ChaCha8Rng>,
    ) -> Self {
        let origin = origin.unwrap_or_else(|| Vector2::new(0.0, 0.0));
        let variance = variance.unwrap_or(1000.0);
        let min_altitude = min_altitude.unwrap_or(-300.0);
        let max_altitude = max_altitude.unwrap_or(-1000.0);

        let (min_altitude, max_altitude) = if min_altitude < max_altitude {
            (min_altitude, max_altitude)
        } else {
            warn!(
                "Invalid altitude range: min_altitude ({}) >= max_altitude ({}). Swapping values.",
                min_altitude, max_altitude
            );
            (max_altitude, min_altitude)
        };

        let mut rng = match rng {
            Some(rng) => rng,
            None => ChaCha8Rng::from_entropy(),
        };

        let u1 = rng.gen::<f64>();
        let u2 = rng.gen::<f64>();
        let radius = variance * (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;

        let x = origin.x + radius * theta.cos();
        let y = origin.y + radius * theta.sin();
        let z = rng.gen_range(min_altitude..max_altitude);

        info!("x: {}, y: {}, z: {}", x, y, z);

        let position = origin.push(0.0) + Vector3::new(x, y, z);
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
