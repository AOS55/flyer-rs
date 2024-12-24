use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum StartupStage {
    BuildUtilities,
    BuildAircraft,
    BuildCameras,
    BuildTerrain,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum UpdateStage {
    UpdateAction,
    UpdateBevy,
    UpdateStates,
}

pub struct StartupSequencePlugin;

impl Plugin for StartupSequencePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Startup,
            (
                StartupStage::BuildUtilities,
                StartupStage::BuildAircraft,
                StartupStage::BuildCameras,
                StartupStage::BuildTerrain,
            )
                .chain(),
        );
    }
}

pub struct UpdateSequencePlugin;

impl Plugin for UpdateSequencePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                UpdateStage::UpdateAction,
                UpdateStage::UpdateBevy,
                UpdateStage::UpdateStates,
            )
                .chain(),
        );
    }
}
