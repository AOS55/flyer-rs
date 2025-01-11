use bevy::prelude::*;

use crate::resources::{AerodynamicsConfig, PhysicsConfig};

pub struct PhysicsPlugin {
    pub config: PhysicsConfig,
}

impl PhysicsPlugin {
    pub fn with_config(config: PhysicsConfig) -> Self {
        Self { config }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Insert PhysicsConfig resource
        app.insert_resource(self.config.clone());

        // Insert AerodynamicsConfig resource
        let aero_config = AerodynamicsConfig {
            min_airspeed_threshold: 0.0,
        };
        app.insert_resource(aero_config);
    }
}
