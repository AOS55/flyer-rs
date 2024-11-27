use crate::world::SimWorld;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tiny_skia::Pixmap;

impl SimWorld {
    pub fn load_terrain_assets(&mut self) {
        let tile_path = self.assets_dir.join("tiles");
        let object_path = self.assets_dir.join("objects");

        self.tile_map = self.load_assets_from_dir(&tile_path);
        self.object_map = self.load_assets_from_dir(&object_path);
    }

    fn load_assets_from_dir(&self, dir_path: &PathBuf) -> HashMap<String, Pixmap> {
        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect::<Vec<_>>(),
            Err(_) => {
                eprintln!("Directory not found: {}", dir_path.display());
                std::process::exit(1);
            }
        };

        let mut asset_map = HashMap::new();
        for path in entries {
            if let Ok(pixmap) = Pixmap::load_png(&path) {
                let name = path.file_stem().unwrap().to_string_lossy().into_owned();
                asset_map.insert(name, pixmap);
            }
        }
        asset_map
    }
}
