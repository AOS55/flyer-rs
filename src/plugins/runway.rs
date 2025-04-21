use bevy::prelude::*;

use crate::{
    components::RunwayComponent,
    plugins::StartupStage,
    systems::spawn_runway_sprite, // System to visually spawn the sprite
};

/// Plugin to manage the runway setup in the simulation.
/// It spawns an entity with the RunwayComponent if configuration is provided.
pub struct RunwayPlugin {
    /// Optional configuration for the runway. If None, no runway is spawned.
    pub config: Option<RunwayComponent>,
}

impl RunwayPlugin {
    /// Creates a new RunwayPlugin.
    pub fn new(config: Option<RunwayComponent>) -> Self {
        Self { config }
    }
}

/// System that spawns the primary runway entity during startup.
fn setup_runway_entity(
    mut commands: Commands,
    runway_config: Option<RunwayComponent>, // Passed in during system registration
) {
    if let Some(config) = runway_config {
        info!("Spawning runway entity with config: {:?}", config);
        commands.spawn(config); // Spawns entity with the RunwayComponent
    } else {
        info!("No RunwayComponent configuration provided, skipping runway entity spawn.");
    }
}

impl Plugin for RunwayPlugin {
    fn build(&self, app: &mut App) {
        let config_clone_for_entity_setup = self.config.clone();

        if self.config.is_some() {
            app.add_systems(
                Startup,
                // Corrected: Apply .in_set() *after* the closure argument
                (move |commands: Commands| {
                    // Clone is needed if the outer config_clone might be used elsewhere,
                    // otherwise the closure takes ownership. Let's keep the clone for safety.
                    let config = config_clone_for_entity_setup.clone();
                    setup_runway_entity(commands, config);
                })
                .in_set(StartupStage::BuildTerrain), // Apply configuration here
            );

            app.add_systems(Update, spawn_runway_sprite);

            info!("RunwayPlugin loaded with configuration.");
        } else {
            info!("RunwayPlugin loaded without configuration (no runway will be spawned).");
        }
    }
}
