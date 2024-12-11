use bevy::prelude::*;
use flyer::components::PhysicsModel;
use flyer::plugins::{AircraftPlugin, CameraPlugin, StartupSet, TransformationPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TransformationPlugin::new(1.0))
        .add_plugins(AircraftPlugin::new(PhysicsModel::Simple))
        .add_plugins(CameraPlugin)
        .configure_sets(
            Startup,
            (StartupSet::SpawnPlayer, StartupSet::SpawnCamera).chain(),
        )
        .run();
}
