use std::collections::HashMap;

use crate::terrain::{Tile, StaticObject, TerrainConfig, Terrain, RandomFuncs};
use crate::aircraft::Aircraft;

use std::{path, env};

use ggez::{ContextBuilder, Context, GameResult, conf};
use ggez::graphics::{self, Image, Color};
use ggez::glam::Vec2;
use ggez::event;

extern crate sdl2;

// use sdl2::pixels::PixelFormatEnum;
// use sdl2::render::Canvas;
// use sdl2::surface::Surface;
// use sdl2::rect::Rect;
// use sdl2::pixels::Color;

use rayon::prelude::*;

pub struct World {
    pub vehicles: Vec<Aircraft>,
    pub camera: Camera,
    pub controls: Vec<f64>,
    pub tiles: Vec<Tile>,
    pub tile_map: HashMap<String, Image>,
    pub objects: Vec<StaticObject>,
    pub object_map: HashMap<String, Image>,
    pub screen_dims: Vec2,
    pub scale: f32,
    pub settings: Settings,
    pub ctx: Context
}

impl Default for World{

    fn default() -> Self {

        // Build a context for the main game window
        let mut cb = ContextBuilder::new("flyer-env", "ggez")
            .window_mode(conf::WindowMode::default().dimensions(1024.0, 1024.0));

        // Add resources to the main game path
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let mut path = path::PathBuf::from(manifest_dir);
            path.push("resources");
            println!("Adding path {path:?}");
            cb = cb.add_resource_path(path);
        }

        let (ctx, _) = cb.build().unwrap();

        Self {
            vehicles: vec![],
            camera: Camera::default(),
            controls: vec![0.0, 0.0, 1.0, 0.0],
            tiles: vec![],
            tile_map: HashMap::new(),
            objects: vec![],
            object_map: HashMap::new(),
            screen_dims: Vec2::new(1024.0, 1024.0),
            scale: 25.0/16.0,
            settings: Settings::default(),
            ctx
        }
    }

}


impl World {

    /// World Construction methods that allow a new world to be setup
    
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
        
        let terrain = Terrain {
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

        let tile_dir: Vec<_> = match self.ctx.fs.read_dir("/tiles") {
            Ok(td) => {
                td.collect()
            },
            Err(_td) => {
                eprintln!("{}", "Tiles dir not found in context");
                std::process::exit(1);
            }
        };

        let so_dir: Vec<_> = match self.ctx.fs.read_dir("/objects") {
            Ok(so) => {
                so.collect()
            },
            Err(_so) => {
                eprintln!("{}", "Object dir not found in context");
                std::process::exit(1);
            }
        };

        let tile_map = terrain.load_assets(&mut self.ctx, tile_dir);
        let object_map = terrain.load_assets(&mut self.ctx, so_dir);

        self.tiles = tiles;
        self.objects = objects;

        self.tile_map = tile_map;
        self.object_map = object_map;

    }

    pub fn set_screen_dims(&mut self,
        width: f32, 
        height: f32,
    ) {
        self.screen_dims = Vec2::new(width, height);
    }

    pub fn add_aircraft(&mut self, aircraft: Aircraft) {
        
        self.vehicles.push(aircraft);
    
    }

}

impl World {

    pub fn render(&mut self) {

        // TODO: Test this out

        self.ctx.gfx.begin_frame().unwrap();  // Start the gfx wgpu frame

        let mut canvas = graphics::Canvas::from_frame(&mut self.ctx, Color::BLACK);
        
        let image = &self.tile_map["grass"];
        let draw_param = graphics::DrawParam::new()
            .dest(Vec2::new(0.0, 0.0))
            .scale(Vec2::new(1.0, 1.0));
        canvas.draw(image, draw_param);

        let center = Vec2::new(self.camera.x as f32, self.camera.y as f32);  // center of image in [m]
        let reconstruction_ratio = self.camera.f * self.camera.z;  // how large the fov is

        let scaling_ratio = Vec2::new(
            self.screen_dims[0] / reconstruction_ratio as f32,
            self.screen_dims[1]/ reconstruction_ratio as f32
        );

        let render_results: Vec<_> = self.tiles.par_iter().filter_map(|tile| {

            let pix_pos = (tile.pos - center) * scaling_ratio;
            let scale = self.scale * scaling_ratio;
            if -50.0 < pix_pos[0]
                && -50.0 < pix_pos[1] 
                && pix_pos[0] < self.screen_dims[0]+50.0
                && pix_pos[1] < self.screen_dims[1]+50.0 {
                    let image = &self.tile_map[&tile.asset];
                    let draw_param = graphics::DrawParam::new()
                        .dest(pix_pos)
                        .scale(scale);
                    Some((image, draw_param))
                } else {
                    None
                }
        }).collect();

        for (image, draw_param) in render_results {
            canvas.draw(image, draw_param);
        }

        let render_results: Vec<_> = self.objects.par_iter().filter_map( |object|{

            let pix_pos = (object.pos - center) * scaling_ratio;
            let scale = self.scale * scaling_ratio;

            if -50.0 < pix_pos[0] 
                && -50.0 < pix_pos[1] 
                && pix_pos[0] < self.screen_dims[0]+50.0 
                && pix_pos[1] < self.screen_dims[1]+50.0 {
                    let image = &self.object_map[&object.asset];
                    let draw_param = graphics::DrawParam::new()
                        .dest(pix_pos)
                        .scale(scale);
                    Some((image, draw_param))
                } else {
                    None
                }
        }).collect();

        for (image, draw_param) in render_results {
            canvas.draw(image, draw_param);
        }

        let _ = canvas.finish(&mut self.ctx);

        let _ = self.ctx.gfx.end_frame();

    } 

    pub fn get_image(&mut self) -> Vec<u8> {

        let frame_image = self.ctx.gfx.frame();
        // println!("Height: {}, Width: {}", frame_image.height(), frame_image.width());
        frame_image.to_pixels(&self.ctx).unwrap()
    
    }

}

impl event::EventHandler<ggez::GameError> for World {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {

        let dt = 0.05;

        for vehicle in &mut self.vehicles {
            vehicle.update(dt, self.controls.clone()); 
        }
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        
        let mut canvas = graphics::Canvas::from_frame(_ctx, Color::BLACK);

        let center = Vec2::new(self.camera.x as f32, self.camera.y as f32);  // center of image in [m]
        let reconstruction_ratio = self.camera.f * self.camera.z;  // how large the fov is
        let scaling_ratio = Vec2::new(
            self.screen_dims[0] / reconstruction_ratio as f32,
            self.screen_dims[1]/ reconstruction_ratio as f32
        );

        let render_results: Vec<_> = self.tiles.par_iter().filter_map(|tile| {

            let pix_pos = (tile.pos - center) * scaling_ratio;
            let scale = self.scale * scaling_ratio;

            if -50.0 < pix_pos[0]
                && -50.0 < pix_pos[1] 
                && pix_pos[0] < self.screen_dims[0]+50.0
                && pix_pos[1] < self.screen_dims[1]+50.0 {
                    let image = &self.tile_map[&tile.asset];
                    let draw_param = graphics::DrawParam::new()
                        .dest(pix_pos)
                        .scale(scale);
                    Some((image, draw_param))
                } else {
                    None
                }
        }).collect();

        for (image, draw_param) in render_results {
            canvas.draw(image, draw_param);
        }

        let render_results: Vec<_> = self.objects.par_iter().filter_map( |object|{

            let pix_pos = (object.pos - center) * scaling_ratio;
            let scale = self.scale * scaling_ratio;

            if -50.0 < pix_pos[0] 
                && -50.0 < pix_pos[1] 
                && pix_pos[0] < self.screen_dims[0]+50.0 
                && pix_pos[1] < self.screen_dims[1]+50.0 {
                    let image = &self.object_map[&object.asset];
                    let draw_param = graphics::DrawParam::new()
                        .dest(pix_pos)
                        .scale(scale);
                    Some((image, draw_param))
                } else {
                    None
                }
        }).collect();

        for (image, draw_param) in render_results {
            canvas.draw(image, draw_param);
        }

        canvas.finish(_ctx) 

    }

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
            z: 1000.0,
            f: 1.0
        }
    }
}

impl Camera {

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
