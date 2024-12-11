use bevy::prelude::*;

use crate::components::aircraft::{
    AircraftConfig, AircraftRenderState, AircraftState, Attitude, DubinsAircraftConfig,
    DubinsAircraftState, PhysicsModel,
};
use crate::components::{AircraftType, PlayerController, SpatialComponent};
use crate::plugins::StartupSet;
use crate::resources::{AircraftAssets, PhysicsConfig};
use crate::systems::{
    aero_force_system, air_data_system, aircraft_render_system, dubins_aircraft_system,
    dubins_keyboard_system, force_calculator_system, physics_integrator_system,
    spawn_aircraft_sprite,
};

pub struct AircraftPlugin {
    physics_model: PhysicsModel,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum SimplePhysicsSet {
    Input,
    Update,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum ComplexPhysicsSet {
    AirData,
    Aerodynamics,
    Forces,
    Integration,
}

impl AircraftPlugin {
    pub fn new(physics_model: PhysicsModel) -> Self {
        Self { physics_model }
    }

    fn setup_dubins_aircraft(mut commands: Commands) {
        commands.spawn((
            DubinsAircraftConfig::default(),
            DubinsAircraftState::default(),
            PlayerController::new(),
            Name::new("Simple Dubins Aircraft"),
            AircraftRenderState {
                attitude: Attitude::Level,
            },
            AircraftType::TwinOtter,
        ));
    }

    fn setup_complex_aircraft(mut commands: Commands) {
        commands.spawn((
            AircraftConfig::default(),
            AircraftState::default(),
            SpatialComponent::default(),
            Name::new("Complex Aircraft"),
        ));
    }

    fn setup_physics_config(mut commands: Commands) {
        commands.insert_resource(PhysicsConfig::default());
    }

    fn setup_assets(
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

impl Plugin for AircraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                move |commands: Commands| Self::setup_physics_config(commands),
                Self::setup_assets,
            )
                .chain(),
        );

        match self.physics_model {
            PhysicsModel::Simple => {
                app.configure_sets(
                    FixedUpdate,
                    (SimplePhysicsSet::Input, SimplePhysicsSet::Update).chain(),
                )
                .add_systems(
                    Startup,
                    (Self::setup_dubins_aircraft, spawn_aircraft_sprite)
                        .chain()
                        .after(Self::setup_assets)
                        .in_set(StartupSet::SpawnPlayer),
                )
                .add_systems(
                    FixedUpdate,
                    (
                        dubins_keyboard_system.in_set(SimplePhysicsSet::Input),
                        dubins_aircraft_system.in_set(SimplePhysicsSet::Update),
                        aircraft_render_system,
                    ),
                );
            }
            PhysicsModel::Full => {
                app.configure_sets(
                    FixedUpdate,
                    (
                        ComplexPhysicsSet::AirData,
                        ComplexPhysicsSet::Aerodynamics,
                        ComplexPhysicsSet::Forces,
                        ComplexPhysicsSet::Integration,
                    )
                        .chain(),
                )
                .add_systems(Startup, Self::setup_complex_aircraft)
                .add_systems(
                    FixedUpdate,
                    (
                        air_data_system.in_set(ComplexPhysicsSet::AirData),
                        aero_force_system.in_set(ComplexPhysicsSet::Aerodynamics),
                        force_calculator_system.in_set(ComplexPhysicsSet::Forces),
                        physics_integrator_system.in_set(ComplexPhysicsSet::Integration),
                    ),
                );
            }
        }

        app.init_resource::<Time<Fixed>>()
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 120.0));
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
