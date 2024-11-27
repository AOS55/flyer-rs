use crate::physics::aerso::AersoState;
use crate::vehicles::traits::VehicleState;
use nalgebra::{UnitQuaternion, Vector3};

#[derive(Debug, Clone)]
pub struct AircraftState {
    // Position in NED frame (North, East, Down) [m]
    pub position: Vector3<f64>,
    // Velocity in body frame [m/s]
    pub velocity: Vector3<f64>,
    // Attitude quaternion (body relative to NED)
    pub attitude: UnitQuaternion<f64>,
    // Angular rates in body frame [rad/s]
    pub rates: Vector3<f64>,
    // Additional state information
    pub air_speed: f64,
    pub ground_speed: f64,
    pub altitude: f64,
    pub heading: f64,
    pub flight_path_angle: f64,
}

impl VehicleState for AircraftState {
    fn position(&self) -> Vector3<f64> {
        self.position
    }

    fn velocity(&self) -> Vector3<f64> {
        self.velocity
    }

    fn attitude(&self) -> UnitQuaternion<f64> {
        self.attitude
    }

    fn rates(&self) -> Vector3<f64> {
        self.rates
    }
}

impl From<&AircraftState> for AersoState {
    fn from(state: &AircraftState) -> Self {
        AersoState {
            position: state.position,
            velocity: state.velocity,
            attitude: state.attitude,
            rates: state.rates,
        }
    }
}

impl From<&AersoState> for AircraftState {
    fn from(state: &AersoState) -> Self {
        AircraftState {
            position: state.position,
            velocity: state.velocity,
            attitude: state.attitude,
            rates: state.rates,
            air_speed: state.velocity.norm(),
            ground_speed: state.velocity.norm(),
            altitude: -state.position.z,
            heading: state.attitude.euler_angles().2,
            flight_path_angle: state.attitude.euler_angles().1,
        }
    }
}
