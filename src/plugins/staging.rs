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

/// StateMachine for the server loop
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum SimState {
    #[default]
    WaitingForAction,
    RunningPhysics,
    SendingResponse,
    Resetting,
}
