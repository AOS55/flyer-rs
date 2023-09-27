// extern crate flyer;
// use flyer::{Terrain, TerrainConfig, Tile, RandomFuncs, StaticObject};
// use flyer::Aircraft;
// use flyer::Camera;

// extern crate nalgebra as na;

// use std::{env, path};
// use std::collections::HashMap;
// use std::{thread, time};

// use rayon::prelude::*;

// use aerso::types::*;

// use ggez::{Context, ContextBuilder, GameResult, conf};
// use ggez::input::keyboard::KeyCode;
// use ggez::graphics::{self, Color, Image};
// use ggez::glam::Vec2;
// use ggez::event;

// #[allow(dead_code)]
// struct MainState {
//     time: f32,
//     step: u32,
//     camera: Camera,
//     aircraft: Aircraft,
//     input: Vec<f64>,
//     tiles: Vec<Tile>,
//     tile_map: HashMap<String, Image>,
//     objects: Vec<StaticObject>,
//     object_map: HashMap<String, Image>,
//     screen_dims: Vec2,
//     scale: Vec2,
//     simulation_frequency: f32,
//     policy_frequency: f32,
//     render_frequency: f32
// }

// impl MainState {
    
//     fn new(ctx: &mut Context) -> GameResult<MainState> {
        
//         let time = 0.0;
//         let step = 0;

//         // let screen = graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb, 1.0, 1.0, 1);
//         let (width, height) = ctx.gfx.drawable_size();
//         let screen_dims = Vec2::new(width, height);

//         let aircraft = Aircraft::new(
//             "TO",
//             Vector3::new(0.0, 0.0, 1000.0),
//             Vector3::new(100.0, 0.0, 0.0),
//             UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
//             Vector3::zeros()
//         );

//         let input = vec![0.0, 0.0, 1.0, 0.0];

//         let t_config = TerrainConfig::default();
//         let seed = 1;
//         let area = vec![256, 256]; 
//         let scaling = 25.0;
        
//         let terrain = Terrain {
//             seed,
//             area,
//             scaling,
//             config: t_config,
//             water_present: true,
//             random_funcs: RandomFuncs::new(seed as u32)
//         };
        
//         let (tiles, objects) = terrain.generate_map();

//         let tile_dir: Vec<_> = ctx.fs.read_dir("/tiles")?.collect();
//         let so_dir: Vec<_> = ctx.fs.read_dir("/objects")?.collect();
//         let tile_map = terrain.load_assets(ctx, tile_dir);
//         let object_map = terrain.load_assets(ctx, so_dir);
//         let scale_x = scaling / 16.0;
//         let scale_y = scaling / 16.0;
//         let scale = Vec2::new(scale_x, scale_y);  // 1:1 scale

//         let camera = Camera {
//             x: 0.0,
//             y: 0.0,
//             z: 1000.0,
//             f: 1.0
//         };

//         let simulation_frequency = 120.0;
//         let policy_frequency = 1.0;
//         let render_frequency = 0.01;

//         let s = MainState {
//             time,
//             step,
//             camera,
//             aircraft,
//             input,
//             tiles,
//             tile_map,
//             objects,
//             object_map,
//             screen_dims,
//             scale,
//             simulation_frequency,
//             policy_frequency,
//             render_frequency,
//         };

//         Ok(s)   
//     }

// }

// impl event::EventHandler<ggez::GameError> for MainState {

//     fn update(&mut self, _ctx: &mut Context) -> GameResult {

//         println!("{}", "Updating!");

//         // Update aircraft based upon inputstate
//         self.aircraft.aff_body.step(0.05, &self.input);

//         Ok(())
//     }

//     fn draw(&mut self, _ctx: &mut Context) -> GameResult {

//         println!("{}", "Drawing!");

//         let mut canvas = graphics::Canvas::from_frame(_ctx, Color::BLACK);

//         let center = Vec2::new(self.camera.x as f32, self.camera.y as f32);  // center of image in [m]
//         let reconstruction_ratio = self.camera.f * self.camera.z;  // how large the fov is
//         let scaling_ratio = Vec2::new(
//             self.screen_dims[0] / reconstruction_ratio as f32,
//             self.screen_dims[1] / reconstruction_ratio as f32
//         );

//         let render_results: Vec<_> = self.tiles.par_iter().filter_map(|tile| {

//             let pix_pos = (tile.pos - center) * scaling_ratio;
//             let scale = self.scale * scaling_ratio;
//             println!("self.scale: {}, scaling_ratio: {}", self.scale, scaling_ratio);
//             if -50.0 < pix_pos[0]
//                 && -50.0 < pix_pos[1] 
//                 && pix_pos[0] < self.screen_dims[0]+50.0
//                 && pix_pos[1] < self.screen_dims[1]+50.0 {
//                     let image = &self.tile_map[&tile.asset];
//                     let draw_param = graphics::DrawParam::new()
//                         .dest(pix_pos)
//                         .scale(scale);
//                     // println!("Scale: {}, PixPos: {}", scale, pix_pos);
//                     Some((image, draw_param))
//                 } else {
//                     None
//                 }
//         }).collect();

//         for (image, draw_param) in render_results {
//             canvas.draw(image, draw_param);
//         }

//         let render_results: Vec<_> = self.objects.par_iter().filter_map( |object|{

//             let pix_pos = (object.pos - center) * scaling_ratio;
//             let scale = self.scale * scaling_ratio;

//             if -50.0 < pix_pos[0] 
//                 && -50.0 < pix_pos[1] 
//                 && pix_pos[0] < self.screen_dims[0]+50.0 
//                 && pix_pos[1] < self.screen_dims[1]+50.0 {
//                     let image = &self.object_map[&object.asset];
//                     let draw_param = graphics::DrawParam::new()
//                         .dest(pix_pos)
//                         .scale(scale);
//                     Some((image, draw_param))
//                 } else {
//                     None
//                 }
//         }).collect();

//         for (image, draw_param) in render_results {
//             canvas.draw(image, draw_param);
//         }

//         canvas.finish(_ctx)   
//     }

//     fn key_down_event(
//             &mut self,
//             _ctx: &mut Context,
//             input: ggez::input::keyboard::KeyInput,
//             _repeated: bool,
//         ) -> Result<(), ggez::GameError> {
        
//         println!("{}", "Key Down?");

//         match input.keycode {
//             Some(KeyCode::Up) => {
//                 self.camera.y -= 10.0;
//             }
//             Some(KeyCode::Down) => {
//                 self.camera.y += 10.0;
//             }
//             Some(KeyCode::Left) => {
//                 self.camera.x -= 10.0;
//             }
//             Some(KeyCode::Right) => {
//                 self.camera.x += 10.0;
//             }
//             Some(KeyCode::PageUp) => {
//                 self.camera.z += 10.0;
//             }
//             Some(KeyCode::PageDown) => {
//                 self.camera.z -= 10.0;
//             }
//             _ => ()
//         }
//         Ok(())
//     }

// }

// fn main() -> GameResult {

//     // Build a context for the main game window
//     let mut cb = ContextBuilder::new("flyer-env", "ggez")
//         .window_mode(conf::WindowMode::default().dimensions(1024.0, 1024.0));
    
//     // Add resources to the main game path
//     if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
//         let mut path = path::PathBuf::from(manifest_dir);
//         path.push("resources");
//         println!("Adding path {path:?}");
//         cb = cb.add_resource_path(path);
//     }

//     // Build the context ctx and event loop
//     let (mut ctx, event_loop) = cb.build()?;

//     // Build the Main Game
//     let state = MainState::new(&mut ctx)?;

//     // println!("Beginning wait!");
//     // let ten_seconds = time::Duration::from_secs(10);
//     // thread::sleep(ten_seconds);

//     // Run the program
//     event::run(ctx, event_loop, state)
// }
