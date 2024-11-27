use super::Terrain;
use super::TerrainData;
use crate::world::SimWorld;
use glam::Vec2;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};

impl SimWorld {
    pub fn create_map(&mut self) {
        let config = &self.settings.terrain_config;

        // Set origin of the map to be in the center
        self.state.origin = Vec2::new(
            self.state.scale * (config.area.0 as f32 / 2.0),
            self.state.scale * (config.area.1 as f32 / 2.0),
        )
        .into();

        let terrain_data = self.load_or_generate_terrain();
        self.state.terrain = Terrain::from_data(terrain_data);

        self.load_terrain_assets();
    }

    fn load_or_generate_terrain(&mut self) -> TerrainData {
        let config = &self.settings.terrain_config;
        let name = format!(
            "seed{}_area0{}1{}_scaling{}_wp{}",
            config.seed, config.area.0, config.area.1, config.scaling, config.water_present
        );

        let config_path = self.terrain_data_dir.join(format!("{}.json", name));

        match fs::File::open(&config_path) {
            Ok(mut file) => {
                let mut json_data = String::new();
                file.read_to_string(&mut json_data)
                    .expect("Failed to read terrain data");
                serde_json::from_str(&json_data).unwrap()
            }
            Err(_) => {
                let terrain_data = self.generate_terrain();
                let serialized = serde_json::to_string(&terrain_data).unwrap();

                fs::create_dir_all(&self.terrain_data_dir).unwrap();
                let mut file = File::create(&config_path).unwrap();
                file.write_all(serialized.as_bytes()).unwrap();

                terrain_data
            }
        }
    }

    fn generate_terrain(&self) -> TerrainData {
        // Original terrain generation code here
        unimplemented!("Terrain generation to be implemented")
    }
}
