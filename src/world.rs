use crate::terrain::{Tile, StaticObject, TerrainConfig, Terrain, RandomFuncs};
use crate::aircraft::Aircraft;
use crate::runway::Runway;

use std::{fs, path::PathBuf};
use std::collections::HashMap;
use std::time::Instant;
use std::io::{Read, Write};
use std::fs::File;

use serde::{Serialize, Deserialize};
use glam::Vec2;
use tiny_skia::*;

use rayon::prelude::*;

#[derive(Serialize, Deserialize)]
struct TerrainData {
    tiles: Vec<Tile>,
    objects: Vec<StaticObject>
}

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
    pub assets_dir: PathBuf,
    pub terrain_data_dir: PathBuf,
    pub runway: Option<Runway>
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
            assets_dir: [r"assets"].iter().collect(),
            terrain_data_dir: [r"terrain_data"].iter().collect(),
            runway: None
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
        

        let now = Instant::now();
        
        // Create the TerrainMap
        let name = terrain.get_name();
        let mut config_path = PathBuf::from(&self.terrain_data_dir);
        config_path.push(name);
        config_path.set_extension("json");
        let (tiles, objects) = match fs::File::open(&config_path) {
            Ok(mut file) => {
                println!("Found File!");
                let mut json_data = String::new();
                file.read_to_string(&mut json_data).expect("Failed to read file");
                println!("read: {:.2?}", now.elapsed());
                let t_data: Result<TerrainData, serde_json::Error> = serde_json::from_str(&json_data);
                println!("serde_json: {:.2?}", now.elapsed());
                let t_data = t_data.unwrap();
                (t_data.tiles, t_data.objects)
            },
            Err(_e) => {
                let (tiles, objects) = terrain.generate_map();
                // TODO: Save the tiles and objects
                let instance = TerrainData {
                    tiles: tiles,
                    objects: objects
                };
                let serialized = serde_json::to_string(&instance).unwrap();
                let mut file = File::create(&config_path).unwrap();
                file.write_all(serialized.as_bytes()).unwrap();

                println!("config_path: {}", config_path.display());
                let mut file = fs::File::open(&config_path).unwrap();
                let mut json_data = String::new(); 
                file.read_to_string(&mut json_data).expect("Failed to read file");
                let t_data: Result<TerrainData, serde_json::Error> = serde_json::from_str(&json_data);
                let t_data = t_data.unwrap();
                (t_data.tiles, t_data.objects)    
            }
        };


        // TODO: Find a way to workout if the map can be loaded from storage or need to generate
        // let (tiles, objects) = terrain.generate_map();
        
         
        println!("generate_map: time: {:.2?}", now.elapsed());
        let now = Instant::now();

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

        println!("Got directory: time: {:.2?}", now.elapsed());

        let tile_map = terrain.load_assets(tile_dir);
        let object_map = terrain.load_assets(so_dir);

        println!("Made maps: time: {:.2?}", now.elapsed());

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

    #[allow(dead_code)]
    pub fn set_terrain_data_dir(&mut self,
        terrain_data_dir: PathBuf
    ) {
        self.terrain_data_dir = terrain_data_dir;
    }

    pub fn create_runway(&mut self) {
        let runway = Runway::default();
        self.runway = Some(runway);
    }

}

impl World {

    pub fn render(&mut self) -> Pixmap {

        // Create the canvas to render onto
        let mut canvas = Pixmap::new(self.screen_dims[0] as u32, self.screen_dims[1] as u32).unwrap();
        let paint = PixmapPaint::default();

        // Calcuate the center of the screen and how much to transform each pixel by
        let center = Vec2::new(self.camera.x as f32 + self.origin[0], self.camera.y as f32 + self.origin[1]);  // center of image in [m]
        let reconstruction_ratio = self.camera.f * self.camera.z;  // how large the fov is
        let scaling_ratio = Vec2::new(
            self.screen_dims[0] / reconstruction_ratio as f32,
            self.screen_dims[1]/ reconstruction_ratio as f32
        );

        // Render tiles
        let render_results: Vec<(Pixmap, Transform)> = self.tiles.par_iter().filter_map(|tile: &Tile| {
            let pos = Vec2::new(tile.pos[0] - center[0], tile.pos[1] - center[1]);
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

        // Render objects 
        let render_results = self.objects.par_iter().filter_map(|object: &StaticObject| {
            let pos = Vec2::new(object.pos[0] - center[0], object.pos[1] - center[1]);
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

        // Render runway if available
        match &self.runway {
            Some(runway) => {
                let mut runway_corner = runway.pos - (runway.dims / 2.0);
                runway_corner[0] = runway_corner[0] - self.camera.x as f32;
                runway_corner[1] = runway_corner[1] - self.camera.y as f32;
                println!("runway.pos: {}, runway.dims: {}, runway_center: {}", runway.pos, runway.dims, runway_corner);
                
    
                let screen_center = self.screen_dims/2.0;

                let pix_pos_corner = runway_corner * scaling_ratio;
                let pix_pos_corner = pix_pos_corner + screen_center;

                let mut runway_center = runway.pos;
                runway_center[0] = runway_center[0] - self.camera.x as f32;
                runway_center[1] = runway_center[1] - self.camera.y as f32;
                

                let pix_pos_center = runway_center * scaling_ratio;
                let pix_pos_center: Vec2 = pix_pos_center + screen_center;
                
                let scale = Vec2::new(scaling_ratio[0] * (runway.dims[0] / 33.0), scaling_ratio[1] * (runway.dims[1] / 1500.0));  // [33.0, 1500.0] comes from the native image dimensions
                let object = &self.object_map[&runway.asset];
                let transform: Transform = Transform::from_row(scale[0], 0.0, 0.0, scale[1], pix_pos_corner[0], pix_pos_corner[1]);
                let transform = transform.post_rotate_at(90.0 + runway.heading, pix_pos_center[0], pix_pos_center[1]);
                canvas.draw_pixmap(0, 0, object.as_ref(), &paint, transform, None);


            },
            None => () 
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
