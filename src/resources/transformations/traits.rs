use crate::resources::Frame;
use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

// Trait for converting positions between different frames
pub trait PositionTransform {
    /// Transform a position from one frame to another
    fn transform_position(
        &self,
        position: &Vector3<f64>,
        from: Frame,
        to: Frame,
    ) -> Result<Vector3<f64>, TransformError>;

    /// Transform a position to screen coordinates (f32)
    fn transform_to_screen_coords(
        &self,
        position: &Vector3<f64>,
        from: Frame,
    ) -> Result<Vec3, TransformError>;
}

/// Trait for converting velocities between different frames
pub trait VelocityTransform {
    /// Transform a velocity from one frame to another
    fn transform_velocity(
        &self,
        velocity: &Vector3<f64>,
        attitude: &UnitQuaternion<f64>,
        from: Frame,
        to: Frame,
    ) -> Result<Vector3<f64>, TransformError>;
}

/// Trait for converting attitudes between different frames
pub trait AttitudeTransform {
    /// Transform an attitude quaternion from one frame to another
    fn transform_attitude(
        &self,
        attitude: &UnitQuaternion<f64>,
        from: Frame,
        to: Frame,
    ) -> Result<UnitQuaternion<f64>, TransformError>;
}

/// Trait for coordinate scaling operations
pub trait ScaleTransform {
    /// Scale a position from meters to pixels
    fn scale_to_pixels(&self, position: &Vector3<f64>) -> Vec3;

    /// Scale a position from pixels to meters
    fn scale_to_meters(&self, position: &Vec3) -> Vector3<f64>;

    /// Get the current scale factor (meters per pixel)
    fn get_scale(&self) -> f64;
}

/// Errors that can occur during transformation
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("Invalid frame transformation from {from:?} to {to:?}")]
    InvalidFrameTransform { from: Frame, to: Frame },

    #[error("Quaternion normalization failed")]
    QuaternionNormalizationError,

    #[error("Scale factor is zero or negative")]
    InvalidScale,
}

/// Bundle of transformation traits that will be implemented by our resource
pub trait TransformationBundle:
    PositionTransform + VelocityTransform + AttitudeTransform + ScaleTransform
{
}
