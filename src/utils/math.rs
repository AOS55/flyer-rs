use nalgebra::{UnitQuaternion, Vector3};
use std::f64::consts::PI;

/// Convert degrees to radians
#[inline]
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

/// Convert radians to degrees
#[inline]
pub fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / PI
}

/// Calculate the flight path angle from a velocity vector
pub fn flight_path_angle(velocity: &Vector3<f64>) -> f64 {
    -velocity
        .z
        .atan2((velocity.x.powi(2) + velocity.y.powi(2)).sqrt())
}

/// Calculate heading from a velocity vector
pub fn heading_from_velocity(velocity: &Vector3<f64>) -> f64 {
    velocity.y.atan2(velocity.x)
}

/// Calculate air data parameters
pub fn calculate_air_data(true_airspeed: f64, alpha: f64, beta: f64) -> (Vector3<f64>, f64, f64) {
    let v_x = true_airspeed * alpha.cos() * beta.cos();
    let v_y = true_airspeed * beta.sin();
    let v_z = true_airspeed * alpha.sin() * beta.cos();

    let velocity = Vector3::new(v_x, v_y, v_z);
    (velocity, alpha, beta)
}

/// Linear interpolation between two values
#[inline]
pub fn lerp(start: f64, end: f64, factor: f64) -> f64 {
    start + (end - start) * factor.clamp(0.0, 1.0)
}

/// Convert a quaternion to Euler angles (roll, pitch, yaw)
pub fn quaternion_to_euler(quat: &UnitQuaternion<f64>) -> Vector3<f64> {
    let (roll, pitch, yaw) = quat.euler_angles();
    Vector3::new(roll, pitch, yaw)
}
