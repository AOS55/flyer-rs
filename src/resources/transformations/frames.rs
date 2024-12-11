use crate::resources::transformations::traits::*;
use bevy::prelude::*;
use nalgebra::{Matrix3, UnitQuaternion, Vector3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    Body,   // Aircraft Body frame: x-forward, y-right, z-down
    NED,    // North-East-Down frame: x-north, y-east, z-down
    Screen, // Bevy render frame: x-right, y-up, z-forward
}

/// Resource that handles all coordinate transformations in the system
#[derive(Resource)]
pub struct TransformationResource {
    /// Scale factor in meters per pixel
    meters_per_pixel: f64,
    /// Rotation matrix from Body to NED
    body_to_ned: Matrix3<f64>,
    /// Rotation matrix from NED to Screen
    ned_to_screen: Matrix3<f64>,
}

impl Default for TransformationResource {
    fn default() -> Self {
        // Initialize with standard rotation matrices
        // Body (x-forward, y-right, z-down) to NED is identity by convention
        let body_to_ned = Matrix3::identity();

        // NED to Screen (up-right-out) requires axis remapping:
        // - NED's Down axis -> Screen's Up axis (negated)
        // - NED's East axis -> Screen's Right axis
        // - NED's North axis -> Screen's Out axis (into screen)
        let ned_to_screen = Matrix3::new(
            0.0, 1.0, 0.0, // X (North) -> Y (Up)
            1.0, 0.0, 0.0, // Y (East)  -> X (Right)
            0.0, 0.0, -1.0, // Z (Down)  -> Z (Out, negated)
        );

        Self {
            meters_per_pixel: 1.0, // Default 1 meter per pixel
            body_to_ned,
            ned_to_screen,
        }
    }
}

impl TransformationResource {
    /// Create a new transformation resource with a specific scale
    pub fn new(meters_per_pixel: f64) -> Result<Self, TransformError> {
        if meters_per_pixel <= 0.0 {
            return Err(TransformError::InvalidScale);
        }
        Ok(Self {
            meters_per_pixel,
            ..Default::default()
        })
    }

    /// Get the transformation matrix between any two frames
    fn get_transform_matrix(&self, from: Frame, to: Frame) -> Result<Matrix3<f64>, TransformError> {
        match (from, to) {
            (Frame::Body, Frame::NED) => Ok(self.body_to_ned),
            (Frame::NED, Frame::Body) => Ok(self.body_to_ned.transpose()),
            (Frame::NED, Frame::Screen) => Ok(self.ned_to_screen),
            (Frame::Screen, Frame::NED) => Ok(self.ned_to_screen.transpose()),
            (Frame::Body, Frame::Screen) => Ok(self.ned_to_screen * self.body_to_ned),
            (Frame::Screen, Frame::Body) => {
                Ok(self.body_to_ned.transpose() * self.ned_to_screen.transpose())
            }
            (f1, f2) if f1 == f2 => Ok(Matrix3::identity()),
            (from, to) => Err(TransformError::InvalidFrameTransform { from, to }),
        }
    }
}

impl PositionTransform for TransformationResource {
    fn transform_position(
        &self,
        position: &Vector3<f64>,
        from: Frame,
        to: Frame,
    ) -> Result<Vector3<f64>, TransformError> {
        let transform = self.get_transform_matrix(from, to)?;
        Ok(transform * position)
    }

    fn transform_to_screen_coords(
        &self,
        position: &Vector3<f64>,
        from: Frame,
    ) -> Result<Vec3, TransformError> {
        // First transform to screen frame
        let screen_pos = self.transform_position(position, from, Frame::Screen)?;

        // Then scale to pixels
        Ok(Vec3::new(
            (screen_pos.x / self.meters_per_pixel) as f32,
            (screen_pos.y / self.meters_per_pixel) as f32,
            (screen_pos.z / self.meters_per_pixel) as f32,
        ))
    }
}

impl VelocityTransform for TransformationResource {
    fn transform_velocity(
        &self,
        velocity: &Vector3<f64>,
        attitude: &UnitQuaternion<f64>,
        from: Frame,
        to: Frame,
    ) -> Result<Vector3<f64>, TransformError> {
        match (from, to) {
            (Frame::Body, Frame::NED) => {
                // For body to NED, we need to rotate by the attitude quaternion
                Ok(attitude * velocity)
            }
            (Frame::NED, Frame::Body) => {
                // For NED to body, rotate by conjugate of attitude
                Ok(attitude.conjugate() * velocity)
            }
            _ => {
                // For other transformations, use the standard transformation matrix
                let transform = self.get_transform_matrix(from, to)?;
                Ok(transform * velocity)
            }
        }
    }
}

impl AttitudeTransform for TransformationResource {
    fn transform_attitude(
        &self,
        attitude: &UnitQuaternion<f64>,
        from: Frame,
        to: Frame,
    ) -> Result<UnitQuaternion<f64>, TransformError> {
        match (from, to) {
            (Frame::Body, Frame::NED) => Ok(*attitude),
            (Frame::NED, Frame::Body) => Ok(attitude.conjugate()),
            (Frame::Body, Frame::Screen) => {
                // Convert through NED
                let ned_attitude = *attitude;
                Ok(UnitQuaternion::from_matrix(&self.ned_to_screen) * ned_attitude)
            }
            (from, to) => Err(TransformError::InvalidFrameTransform { from, to }),
        }
    }
}

impl ScaleTransform for TransformationResource {
    fn scale_to_pixels(&self, position: &Vector3<f64>) -> Vec3 {
        Vec3::new(
            (position.x / self.meters_per_pixel) as f32,
            (position.y / self.meters_per_pixel) as f32,
            (position.z / self.meters_per_pixel) as f32,
        )
    }

    fn scale_to_meters(&self, position: &Vec3) -> Vector3<f64> {
        Vector3::new(
            position.x as f64 * self.meters_per_pixel,
            position.y as f64 * self.meters_per_pixel,
            position.z as f64 * self.meters_per_pixel,
        )
    }

    fn get_scale(&self) -> f64 {
        self.meters_per_pixel
    }
}

impl TransformationResource {
    pub fn screen_from_ned(&self, ned_pos: &Vector3<f64>) -> Result<Vec3, TransformError> {
        // Transform and scale in one operation
        let screen_pos = self.transform_position(ned_pos, Frame::NED, Frame::Screen)?;
        Ok(self.scale_to_pixels(&screen_pos))
    }

    pub fn ned_from_body(&self, body_pos: &Vector3<f64>) -> Result<Vector3<f64>, TransformError> {
        self.transform_position(body_pos, Frame::Body, Frame::NED)
    }

    pub fn screen_from_body(&self, body_pos: &Vector3<f64>) -> Result<Vec3, TransformError> {
        let ned_pos = self.ned_from_body(body_pos)?;
        self.screen_from_ned(&ned_pos)
    }
}

impl TransformationBundle for TransformationResource {}
