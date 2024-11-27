use nalgebra::{UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleState {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub attitude: UnitQuaternion<f64>,
    pub angular_velocity: Vector3<f64>,
    pub time: f64,
}

impl Default for VehicleState {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
            time: 0.0,
        }
    }
}

impl VehicleState {
    pub fn new(
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        attitude: UnitQuaternion<f64>,
        angular_velocity: Vector3<f64>,
    ) -> Self {
        Self {
            position,
            velocity,
            attitude,
            angular_velocity,
            time: 0.0,
        }
    }

    pub fn position(&self) -> Vector3<f64> {
        self.position
    }

    pub fn velocity(&self) -> Vector3<f64> {
        self.velocity
    }

    pub fn attitude(&self) -> UnitQuaternion<f64> {
        self.attitude
    }

    pub fn angular_velocity(&self) -> Vector3<f64> {
        self.angular_velocity
    }

    pub fn get_euler_angles(&self) -> Vector3<f64> {
        let (roll, pitch, yaw) = self.attitude.euler_angles();
        Vector3::new(roll, pitch, yaw)
    }

    pub fn set_position(&mut self, position: Vector3<f64>) {
        self.position = position;
    }

    pub fn set_velocity(&mut self, velocity: Vector3<f64>) {
        self.velocity = velocity;
    }

    pub fn set_attitude(&mut self, attitude: UnitQuaternion<f64>) {
        self.attitude = attitude;
    }

    pub fn set_angular_velocity(&mut self, angular_velocity: Vector3<f64>) {
        self.angular_velocity = angular_velocity;
    }

    pub fn advance_time(&mut self, dt: f64) {
        self.time += dt;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Acceleration {
    pub linear: Vector3<f64>,
    pub angular: Vector3<f64>,
}

impl Default for Acceleration {
    fn default() -> Self {
        Self {
            linear: Vector3::zeros(),
            angular: Vector3::zeros(),
        }
    }
}
