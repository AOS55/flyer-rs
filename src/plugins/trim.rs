use bevy::prelude::*;
use crate::components::TrimRequest;
use crate::components::TrimSolverConfig;

pub struct TrimPlugin;

impl Plugin for TrimPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TrimRequest>() 
           .init_resource::<TrimSolverConfig>(); 
        // Add event handler
        // Trim system itself is added in FixedUpdate in main()
    }
}
