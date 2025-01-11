use bevy::prelude::*;
use bevy::window::WindowPlugin;

use flyer::components::{AircraftConfig, DubinsAircraftConfig};
use flyer::plugins::{add_aircraft_plugin, CameraPlugin, TerrainPlugin, TransformationPlugin};
use flyer::resources::terrain::TerrainConfig;

/// Example demonstrating how to set up and run an aircraft with an agent
fn main() {
    let mut app = App::new();

    // 1. Core Bevy plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Aircraft Agent Example".into(),
            resolution: (1280., 720.).into(),
            ..default()
        }),
        ..default()
    }));

    // 2. Setup coordinate transformation system
    app.add_plugins(TransformationPlugin::new(1.0));

    // 3. Setup terrain generation
    app.add_plugins(TerrainPlugin::with_config(TerrainConfig { ..default() }));

    // 4. Setup Dubins aircraft (simplified physics)
    setup_dubins_aircraft(&mut app);
    app.add_plugins(CameraPlugin);

    // 5. Configure startup sets to ensure proper initialization order
    // app.configure_sets(
    //     Startup,
    //     (StartupSet::SpawnPlayer, StartupSet::SpawnCamera).chain(),
    // );

    // 6. Run the simulation
    app.run();
}

/// Configure a Dubins aircraft (simplified physics)
fn setup_dubins_aircraft(app: &mut App) {
    // Create Dubins aircraft configuration
    let config = DubinsAircraftConfig {
        name: "Dubins Agent Aircraft".into(),
        max_speed: 50.0, // meters per second
        min_speed: 20.0,
        max_turn_rate: 0.5,  // radians per second
        max_climb_rate: 5.0, // meters per second
        ..default()
    };

    // Create plugin with Dubins configuration
    add_aircraft_plugin(app, AircraftConfig::Dubins(config));
}
