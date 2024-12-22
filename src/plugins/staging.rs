use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum StartupStage {
    BuildUtilities,
    BuildAircraft,
    BuildCameras,
    BuildTerrain,
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
