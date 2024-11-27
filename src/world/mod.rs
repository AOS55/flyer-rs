pub mod asset_management;
pub mod rendering;
pub mod settings;
pub mod state;
pub mod terrain_management;
pub mod traits;
pub mod world_core;

use std::collections::HashMap;
use std::path::PathBuf;

use tiny_skia::*;

use crate::environment::{Runway, Terrain, TerrainConfig};
use crate::rendering::RenderType;

pub use settings::SimulationSettings;
pub use state::WorldState;
pub use traits::{World, WorldSettings};
pub mod camera;

pub struct SimWorld {
    state: WorldState,
    settings: SimulationSettings,
    assets_dir: PathBuf,
    terrain_data_dir: PathBuf,
    tile_map: HashMap<String, Pixmap>,
    object_map: HashMap<String, Pixmap>,
}

impl SimWorld {
    pub fn new(settings: SimulationSettings) -> Self {
        let state = WorldState::new(
            (
                settings.render_config.screen_width,
                settings.render_config.screen_height,
            ),
            settings.render_config.scale,
        );

        Self {
            state,
            settings,
            assets_dir: PathBuf::from("assets"),
            terrain_data_dir: PathBuf::from("terrain_data"),
            tile_map: HashMap::new(),
            object_map: HashMap::new(),
        }
    }
}
