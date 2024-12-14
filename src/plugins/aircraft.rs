use bevy::prelude::*;

use crate::components::aircraft::Attitude;
use crate::components::{AircraftConfig, AircraftType};
use crate::plugins::{DubinsAircraftPlugin, FullAircraftPlugin};
use crate::resources::AircraftAssets;

pub struct AircraftPluginBase {
    pub config: AircraftConfig,
}

// Update loop for DubinsAircraft
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimplePhysicsSet {
    Input,
    Update,
}

// Update loop for FullAircraft
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ComplexPhysicsSet {
    AirData,
    Aerodynamics,
    Forces,
    Integration,
}

impl AircraftPluginBase {
    pub fn new(config: AircraftConfig) -> Self {
        Self { config }
    }

    pub fn setup_assets(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut sprite_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) {
        info!("Setting up aircraft assets...");
        let sprite_layout = TextureAtlasLayout::from_grid(UVec2::new(128, 128), 3, 3, None, None);
        let layout_handle = sprite_layouts.add(sprite_layout);

        let mut aircraft_assets = AircraftAssets::new();

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

        setup_attitude_mappings(&mut aircraft_assets);
        commands.insert_resource(aircraft_assets);
        info!("Aircraft assets setup complete!");
    }
}

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

    for (attitude, index) in aircraft_mappings {
        aircraft_assets.aircraft_mappings.insert(attitude, index);
    }
}

pub fn add_aircraft_plugin(app: &mut App, config: AircraftConfig) -> &mut App {
    match config {
        AircraftConfig::Full(config) => app.add_plugins(FullAircraftPlugin::new(config)),
        AircraftConfig::Dubins(config) => app.add_plugins(DubinsAircraftPlugin::new(config)),
    }
}
