mod config;
mod generator;
mod tile;
mod utils;

pub use config::TerrainConfig;
pub use generator::TerrainGenerator;
pub use tile::{StaticObject, Tile};
pub use utils::RandomFuncs;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tiny_skia::Pixmap;

#[derive(Serialize, Deserialize)]
pub struct TerrainData {
    pub tiles: Vec<Tile>,
    pub objects: Vec<StaticObject>,
}

pub struct Terrain {
    pub generator: TerrainGenerator,
    pub tiles: Vec<Tile>,
    pub objects: Vec<StaticObject>,
    pub tile_map: HashMap<String, Pixmap>,
    pub object_map: HashMap<String, Pixmap>,
}

impl Default for Terrain {
    fn default() -> Self {
        Self::new(
            0,
            vec![100, 100],
            1.0,
            TerrainConfig::default(),
            false,
            PathBuf::from("assets"),
            PathBuf::from("terrain_data"),
        )
    }
}

impl Terrain {
    pub fn new(
        seed: u64,
        area: Vec<usize>,
        scaling: f32,
        config: TerrainConfig,
        water_present: bool,
        assets_dir: PathBuf,
        terrain_data_dir: PathBuf,
    ) -> Self {
        let generator = TerrainGenerator::new(seed, area, scaling, config, water_present);
        let (tiles, objects) = generator.generate_or_load_map(&terrain_data_dir);
        let tile_map = generator.load_tile_assets(&assets_dir);
        let object_map = generator.load_object_assets(&assets_dir);

        Self {
            generator,
            tiles,
            objects,
            tile_map,
            object_map,
        }
    }

    pub fn width(&self) -> usize {
        self.generator.area[0]
    }

    pub fn height(&self) -> usize {
        self.generator.area[1]
    }
}
