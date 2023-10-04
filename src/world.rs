use std::collections::HashMap;

use crate::terrain::{Tile, StaticObject, TerrainConfig, Terrain, RandomFuncs};
use crate::aircraft::Aircraft;

use std::{fs, path::PathBuf};

use glam::Vec2;
use tiny_skia::*;

use rayon::prelude::*;

pub struct World {
    pub vehicles: Vec<Aircraft>,
    pub camera: Camera,
    pub controls: Vec<f64>,
    pub tiles: Vec<Tile>,
    pub tile_map: HashMap<String, Pixmap>,
    pub objects: Vec<StaticObject>,
    pub object_map: HashMap<String, Pixmap>,
    pub screen_dims: Vec2,
    pub scale: f32,
    origin: Vec2,
    pub settings: Settings,
    pub assets_dir: PathBuf
}

impl Default for World{

    fn default() -> Self {

        Self {
            vehicles: vec![],
            camera: Camera::default(),
            controls: vec![0.0, 0.0, 1.0, 0.0],
            tiles: vec![],
            tile_map: HashMap::new(),
            objects: vec![],
            object_map: HashMap::new(),
            screen_dims: Vec2::new(1024.0, 1024.0),
            scale: 25.0,
            origin: Vec2::new(0.0, 0.0),
            settings: Settings::default(),
            assets_dir: [r"assets"].iter().collect()
        }
    }

}


impl World {

    /// World Construction methods that allow a new world to be setup

    #[allow(dead_code)]
    pub fn update_settings(&mut self,
        simulation_frequency: Option<f64>,
        policy_frequency: Option<f64>,
        render_frequency: Option<f64>
    ) {
        self.settings = Settings::new(
            simulation_frequency,
            policy_frequency,
            render_frequency
        )
    }

    pub fn create_map(&mut self,
        seed: u64,
        area: Option<Vec<usize>>,
        scaling: Option<f32>,
        water_present: Option<bool>
    ) {
        // Build from default, if there are other values we deal with those
        let terrain_config = TerrainConfig::default();

        let area = if let Some(area) = area {
                area
            } else {
                vec![256, 256]
            };

        let scaling = if let Some(scale) = scaling {
            scale
        } else {
            self.scale
        };

        let water_present = if let Some(water_present) = water_present {
            water_present
        } else {
            false
        };


        // Set origin of the map to be in the center
        self.origin = Vec2::new(self.scale * (area[0] as f32 / 2.0), self.scale * (area[1] as f32 / 2.0));
        
        let mut terrain = Terrain {
            seed,
            area,
            scaling,
            config: terrain_config,
            water_present,
            random_funcs: RandomFuncs::new(seed as u32)
        };

        // Create the TerrainMap
        // TODO: Find a way to workout if the map can be loaded from storage or need to generate
        let (tiles, objects) = terrain.generate_map();

        // Build up the TileMap from the context fs, only part that uses ctx from GGEZ

        let mut path = PathBuf::from(&self.assets_dir);
        path.push("tiles");

        let tile_dir: Vec<_> = match fs::read_dir(&path) {
            Ok(td) => {
                td
                .filter_map(|entry| Some(entry.ok()?.path()))
                .collect()
            },
            Err(_td) => {
                eprintln!("Tiles dir not found in context, path is: {}", path.as_path().display());
                std::process::exit(1);
            }
        };
        
        let mut path = PathBuf::from(&self.assets_dir);
        path.push("objects");

        let so_dir: Vec<_> = match fs::read_dir(&path) {
            Ok(so) => {
                so
                .filter_map(|entry| Some(entry.ok()?.path()))
                .collect()
            },
            Err(_so) => {
                eprintln!("Object dir not found in context, path is: {}", path.as_path().display());
                std::process::exit(1);
            }
        };

        let tile_map = terrain.load_assets(tile_dir);
        let object_map = terrain.load_assets(so_dir);

        self.tiles = tiles;
        self.objects = objects;

        self.tile_map = tile_map;
        self.object_map = object_map;

    }

    #[allow(dead_code)]
    pub fn set_screen_dims(&mut self,
        width: f32, 
        height: f32,
    ) {
        self.screen_dims = Vec2::new(width, height);
    }

    #[allow(dead_code)]
    pub fn add_aircraft(&mut self, aircraft: Aircraft) {

        self.vehicles.push(aircraft);
    
    }

    #[allow(dead_code)]
    pub fn set_assets_dir(&mut self,
        assets_dir: PathBuf
    ) {
        self.assets_dir = assets_dir;
    }

}

impl World {

    pub fn render(&mut self) -> Pixmap {

        // TODO: Test this out

        let mut canvas = Pixmap::new(self.screen_dims[0] as u32, self.screen_dims[1] as u32).unwrap();
        let paint = PixmapPaint::default();

        println!("Origin: {}", &self.origin);

        let center = Vec2::new(self.camera.x as f32 + self.origin[0], self.camera.y as f32 + self.origin[1]);  // center of image in [m]
        let reconstruction_ratio = self.camera.f * self.camera.z;  // how large the fov is

        let scaling_ratio = Vec2::new(
            self.screen_dims[0] / reconstruction_ratio as f32,
            self.screen_dims[1]/ reconstruction_ratio as f32
        );

        let render_results: Vec<(Pixmap, Transform)> = self.tiles.par_iter().filter_map(|tile: &Tile| {
            let pos = tile.pos - center;
            let pix_pos = pos * scaling_ratio;
            let pix_pos = pix_pos + self.screen_dims/2.0;
            let scale = self.scale * scaling_ratio;
            if -50.0 < pix_pos[0]
                && -50.0 < pix_pos[1] 
                && pix_pos[0] < self.screen_dims[0]+50.0
                && pix_pos[1] < self.screen_dims[1]+50.0 {
                    let tile = &self.tile_map[&tile.asset];
                    let transform = Transform::from_row(scale[0]/16.0, 0.0, 0.0, scale[1]/16.0, pix_pos[0], pix_pos[1]);
                    Some((tile.clone(), transform))
                } else {
                    None
                }
        }).collect::<Vec<(Pixmap, Transform)>>();

        for (pixmap, transform) in render_results {
            canvas.draw_pixmap(0, 0, pixmap.as_ref(), &paint, transform, None);
        }

        let render_results = self.objects.par_iter().filter_map(|object: &StaticObject| {
            let pos = object.pos - center;
            let pix_pos = pos * scaling_ratio;
            let pix_pos = pix_pos + self.screen_dims/2.0;
            let scale = self.scale * scaling_ratio;
            if -50.0 < pix_pos[0]
                && -50.0 < pix_pos[1] 
                && pix_pos[0] < self.screen_dims[0]+50.0
                && pix_pos[1] < self.screen_dims[1]+50.0 {
                    let object = &self.object_map[&object.asset];
                    let transform: Transform = Transform::from_row(scale[0]/16.0, 0.0, 0.0, scale[1]/16.0, pix_pos[0], pix_pos[1]);
                    Some((object.clone(), transform))
                } else {
                    None
                }
        }).collect::<Vec<(Pixmap, Transform)>>();

        for (pixmap, transform) in render_results {
            canvas.draw_pixmap(0, 0, pixmap.as_ref(), &paint, transform, None);
        }

        canvas

    } 

    // pub fn get_image(&mut self) -> Vec<u8> {

    //     let rect = Rect::new(0, 0, 1024, 1024);
    //     self.canvas.read_pixels(rect, PixelFormatEnum::RGB24).unwrap()
    
    // }

}


pub struct Camera {
    pub x: f64,  // camera's x-position
    pub y: f64,  // camera's y-position
    pub z: f64,  // camera's z-position
    pub f: f64,  // camera's reconstruction ratio/zoom
}

impl Default for Camera{

    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0, 
            z: 5000.0,
            f: 1.0
        }
    }
}

impl Camera {

    #[allow(dead_code)]
    fn new(
        x: f64,
        y: f64,
        z: f64,
        f: Option<f64>
    ) -> Self {

        let f = if let Some(focal_dist) = f {
            focal_dist
        } else {
            1.0
        };

        Self {
            x,
            y,
            z,
            f
        }

    }

    pub fn move_camera(&mut self, pos: Vec<f64>) {
        self.x = pos[0];
        self.y = pos[1];
        self.z = -1.0 * pos[2];
    }

}

pub struct Settings {
    pub simulation_frequency: f64,  // frequency of simulation update [Hz]
    pub policy_frequency: f64,  // frequency of policy update
    pub render_frequency: f64,  // frequency of render update
}

impl Default for Settings {

    fn default() -> Self {
        Self {
            simulation_frequency: 120.0,
            policy_frequency: 1.0,
            render_frequency: 0.01,
        }
    }
}

impl Settings {

    fn new(
        simulation_frequency: Option<f64>,
        policy_frequency: Option<f64>,
        render_frequency: Option<f64>
    ) -> Self {

        let simulation_frequency = if let Some(frequency) = simulation_frequency {
            frequency
        } else {
            120.0
        };

        let policy_frequency = if let Some(frequency) = policy_frequency {
            frequency
        } else {
            1.0
        };

        let render_frequency = if let Some(frequency) = render_frequency {
            frequency
        } else {
            0.01
        };

        Self {
            simulation_frequency,
            policy_frequency,
            render_frequency
        }
    }

}
