use bevy::prelude::*;

use crate::components::aircraft::Attitude;
use crate::components::{AircraftConfig, AircraftType};
use crate::plugins::{DubinsAircraftPlugin, FullAircraftPlugin};
use crate::resources::AircraftAssets;

/// Base plugin for initializing aircraft systems and managing assets.
/// It serves as a common setup for both simple (Dubins) and complex (Full) aircraft plugins.
pub struct AircraftPluginBase {
    /// Aircraft configuration that determines whether Dubins or Full physics is used.
    pub config: AircraftConfig,
}

#[derive(Resource)]
pub struct AircraftBaseInitialized;

// System sets for the FullAircraft (complex physics)
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ComplexPhysicsSet {
    /// Calculation of air data such as airspeed, angle of attack, etc.
    AirData,
    /// Aerodynamic force and moment calculations.
    Aerodynamics,
    /// Accumulation of external forces and moments.
    Forces,
    /// Physics integration to update position, velocity, and orientation.
    Integration,
}

impl AircraftPluginBase {
    /// Creates a new `AircraftPluginBase` with the provided configuration.
    ///
    /// # Arguments:
    /// * `config` - The aircraft configuration, either `Full` or `Dubins`.
    pub fn new(config: AircraftConfig) -> Self {
        Self { config }
    }

    /// Sets up the aircraft assets, including textures and sprite layouts, and inserts them as a resource.
    ///
    /// # Arguments:
    /// * `commands` - Used to insert the `AircraftAssets` resource into the Bevy app.
    /// * `asset_server` - The Bevy asset server for loading textures.
    /// * `sprite_layouts` - Handles for managing texture atlas layouts.
    pub fn setup_assets(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut sprite_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) {
        info!("Setting up aircraft assets...");
        // Define the texture atlas layout (3x3 grid, each cell 128x128 pixels)
        let sprite_layout = TextureAtlasLayout::from_grid(UVec2::new(128, 128), 3, 3, None, None);
        let layout_handle = sprite_layouts.add(sprite_layout);

        let mut aircraft_assets = AircraftAssets::new();

        // Load textures and associate layouts for each aircraft type
        for ac_type in [
            AircraftType::TwinOtter,
            AircraftType::F4Phantom,
            AircraftType::GenericTransport,
        ] {
            aircraft_assets.aircraft_textures.insert(
                ac_type.clone(),
                asset_server.load(ac_type.get_texture_path()),
            );
            aircraft_assets
                .aircraft_layouts
                .insert(ac_type, layout_handle.clone());
        }

        // Map sprite indices to different aircraft attitudes (e.g., Level, Up, Down)
        setup_attitude_mappings(&mut aircraft_assets);

        // Insert the aircraft assets as a resource into the ECS
        commands.insert_resource(aircraft_assets);

        info!("Aircraft assets setup complete!");
    }
}

/// Maps aircraft attitudes to corresponding texture atlas indices.
///
/// # Arguments:
/// * `aircraft_assets` - Mutable reference to the `AircraftAssets` resource.
fn setup_attitude_mappings(aircraft_assets: &mut AircraftAssets) {
    let aircraft_mappings = [
        (Attitude::UpRight, 0),
        (Attitude::Right, 1),
        (Attitude::DownRight, 2),
        (Attitude::Up, 3),
        (Attitude::Level, 4),
        (Attitude::Down, 5),
        (Attitude::UpLeft, 6),
        (Attitude::LevelLeft, 7),
        (Attitude::DownLeft, 8),
    ];

    // Insert the mappings into the aircraft assets
    for (attitude, index) in aircraft_mappings {
        aircraft_assets.aircraft_mappings.insert(attitude, index);
    }
}

/// Adds the appropriate aircraft plugin (Full or Dubins) to the Bevy application.
///
/// # Arguments:
/// * `app` - Mutable reference to the Bevy application.
/// * `config` - The aircraft configuration determining which plugin to load.
///
/// # Returns:
/// A mutable reference to the Bevy application.
pub fn add_aircraft_plugin(app: &mut App, config: AircraftConfig) -> &mut App {
    info!("Adding aircraft plugin... {:?}", config);
    match config {
        // Add the FullAircraftPlugin for detailed physics simulation
        AircraftConfig::Full(config) => app.add_plugins(FullAircraftPlugin::new_single(config)),
        // Add the DubinsAircraftPlugin for simplified physics simulation
        AircraftConfig::Dubins(config) => app.add_plugins(DubinsAircraftPlugin::new_single(config)),
    }
}
