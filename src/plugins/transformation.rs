use bevy::prelude::*;

use crate::resources::TransformationResource;

/// Plugin that sets up the coordinate transformation system
pub struct TransformationPlugin {
    meters_per_pixel: f64,
}

impl TransformationPlugin {
    /// Create a new plugin with the given scale
    pub fn new(meters_per_pixel: f64) -> Self {
        Self { meters_per_pixel }
    }

    /// Create a new plugin with default settings
    pub fn default() -> Self {
        Self {
            meters_per_pixel: 1.0, // Default scale
        }
    }
}

impl Plugin for TransformationPlugin {
    fn build(&self, app: &mut App) {
        // Add the transformation resource
        match TransformationResource::new(self.meters_per_pixel) {
            Ok(resource) => {
                app.insert_resource(resource);
            }
            Err(e) => {
                error!(
                    "Failed to create transformation resource with scale: {:?}. Defaulting to scale = 1.0",
                    e
                );
                app.insert_resource(TransformationResource::default());
            }
        }
    }
}
