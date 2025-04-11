use bevy::prelude::*;
use crate::components::TrimRequest;
use crate::systems::handle_trim_requests;

pub struct TrimPlugin;

impl Plugin for TrimPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TrimRequest>() // Register event
            .add_systems(Update, handle_trim_requests);
        // Add event handler
        // Trim system itself is added in FixedUpdate in main()
    }
}
