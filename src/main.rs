mod terrain;
use terrain::{Terrain, TerrainConfig, Tile, RandomFuncs, StaticObject};

mod utils;
extern crate nalgebra as na;

use std::{env, path};
use std::collections::HashMap;
use std::time::{self, Instant};

use rayon::prelude::*;

use ggez::{Context, ContextBuilder, GameResult, conf};
use ggez::input::keyboard::KeyCode;
use ggez::graphics::{self, Color, Image};
use ggez::glam::Vec2;
use ggez::event;

struct Camera {
    x: f32,  // camera's x-position
    y: f32,  // camera's y-position
    z: f32,  // camera's z-position
    f: f32   // camera's reconstruction ratio/zoom
}

struct MainState {
    camera: Camera,
    tiles: Vec<Tile>,
    tile_map: HashMap<String, Image>,
    objects: Vec<StaticObject>,
    object_map: HashMap<String, Image>,
    screen: graphics::ScreenImage,
    screen_dims: Vec2,
    scale: Vec2,
}

impl MainState {
    
    fn new(ctx: &mut Context) -> GameResult<MainState> {
    
        let screen = graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb, 1.0, 1.0, 1);
        let (width, height) = ctx.gfx.drawable_size();
        let screen_dims = Vec2::new(width, height);

        let t_config = TerrainConfig::default();
        
        let seed = 1;
        let area = vec![256, 256]; 
        // let scaling = width / (area[0] as f32);
        let scaling = 25.0;
        
        let terrain = Terrain {
            seed,
            area,
            scaling,
            config: t_config,
            water_present: true,
            random_funcs: RandomFuncs::new(seed as u32)
        };
        
        let (tiles, objects) = terrain.generate_map();
        // for tile in tiles.iter() {
        //     // println!("tile: name is {}, with asset {} at pos {}", tile.name, tile.asset, tile.pos);
        // }
        // for object in objects.iter() {
        //     println!("object: name is {}, with asset {} at pos {}", object.name, object.asset, object.pos);
        // }

        let tile_dir: Vec<_> = ctx.fs.read_dir("/tiles")?.collect();
        let so_dir: Vec<_> = ctx.fs.read_dir("/objects")?.collect();
        let tile_map = terrain.load_assets(ctx, tile_dir);
        let object_map = terrain.load_assets(ctx, so_dir);
        // let land_map = terrain.generate_land_map();

        for key in object_map.keys() {
            println!("{}", key);
        }
    
        // let scale_x = width / (16.0 * terrain.area[0] as f32);
        // let scale_y = height / (16.0 * terrain.area[1] as f32);
        let scale_x = scaling / 16.0;
        let scale_y = scaling / 16.0;
        let scale = Vec2::new(scale_x, scale_y);  // 1:1 scale

        let camera = Camera {
            x: 0.0,
            y: 0.0,
            z: 1000.0,
            f: 1.0
        };

        let s = MainState {
            camera,
            tiles,
            tile_map,
            objects,
            object_map,
            screen,
            screen_dims,
            scale
        };

        Ok(s)   
    }

}

impl event::EventHandler<ggez::GameError> for MainState {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {

        let mut canvas = graphics::Canvas::from_frame(_ctx, Color::BLACK);

        let center = Vec2::new(self.camera.x, self.camera.y);  // center of image in [m]
        let reconstruction_ratio = self.camera.f * self.camera.z;  // how large the fov is
        let scaling_ratio = Vec2::new(
            self.screen_dims[0] / reconstruction_ratio,
            self.screen_dims[1]/ reconstruction_ratio
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

    fn key_down_event(
            &mut self,
            ctx: &mut Context,
            input: ggez::input::keyboard::KeyInput,
            _repeated: bool,
        ) -> Result<(), ggez::GameError> {
        match input.keycode {
            Some(KeyCode::Up) => {
                self.camera.y -= 10.0;
            }
            Some(KeyCode::Down) => {
                self.camera.y += 10.0;
            }
            Some(KeyCode::Left) => {
                self.camera.x -= 10.0;
            }
            Some(KeyCode::Right) => {
                self.camera.x += 10.0;
            }
            Some(KeyCode::PageUp) => {
                self.camera.z += 10.0;
            }
            Some(KeyCode::PageDown) => {
                self.camera.z -= 10.0;
            }
            _ => ()
        }
        Ok(())
    }

}

fn main() -> GameResult {

    let mut cb = ContextBuilder::new("flyer-env", "ggez")
        .window_mode(conf::WindowMode::default().dimensions(400.0, 400.0));
    
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {path:?}");
        cb = cb.add_resource_path(path);
    }

    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)

}
