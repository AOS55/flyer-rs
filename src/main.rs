use ggez::{Context, ContextBuilder, GameResult, conf};
use ggez::graphics::{self, Color, Image};
use aerso::types::*;
use image::{DynamicImage, ImageBuffer};

// use ggez::event::{self, EventHandler};

mod world;
mod aircraft;
mod terrain;

use plotters::style::RGBAColor;
use world::World;
use aircraft::Aircraft;

use std::fs::File;
use std::{env, path};

/// Render the image
fn render(_ctx: &mut Context) -> GameResult {
    
    _ctx.gfx.begin_frame().unwrap();
    let image = Image::from_color(
        _ctx,
        12,
        12, 
        Some(Color::BLUE));

    // let canvas = graphics::Canvas::from_image(_ctx, image, Color::BLACK);
    let mut canvas = graphics::Canvas::from_frame(_ctx, Color::RED);
    
    canvas.draw(&image, ggez::glam::Vec2::new(0.0, 0.0));

    canvas.finish(_ctx)?;
    _ctx.gfx.end_frame()?;

    Ok(())
    
}

/// Use wgpu gfx context to retrieve the image data as serialized [RGBA] data
fn get_image(_ctx: &mut Context) -> GameResult<Vec<u8>> {

    let frame_image = _ctx.gfx.frame();
    // println!("Height: {}, Width: {}", frame_image.height(), frame_image.width());
    let pixels = frame_image.to_pixels(_ctx)?;
    println!("{:?}", pixels);

    Ok(pixels)

}

fn main() {

    // // Build a context for the main game window
    // let mut cb = ContextBuilder::new("flyer-env", "ggez")
    //     .window_mode(conf::WindowMode::default().dimensions(64.0, 64.0));

    // // Add resources to the main game path
    // if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
    //     let mut path = path::PathBuf::from(manifest_dir);
    //     path.push("resources");
    //     println!("Adding path {path:?}");
    //     cb = cb.add_resource_path(path);
    // }

    let aircraft = Aircraft::new(
        "TO",
        Vector3::new(0.0, 0.0, 1000.0),
        Vector3::new(100.0, 0.0, 0.0),
        UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
        Vector3::zeros()
    );

    // let (mut ctx, _) = cb.build().unwrap();

    let mut w = World::default();
    w.add_aircraft(aircraft);
    w.create_map(
        1,
        Some(vec![256, 256]),
        Some(25.0),
        Some(true) 
    );

    w.render();
    let bytes = w.get_image();
    let buffer: ImageBuffer<image::Bgra<_>, Vec<u8>> = ImageBuffer::from_raw(
        1024, 
        1024, 
        bytes).expect("Failed to create ImageBuffer");

    let dynamic_image: DynamicImage = DynamicImage::ImageBgra8(buffer);
    dynamic_image.save("image.jpg").expect("Failed to save image");

    w.ctx.quit_requested = true;

    let w = World::default();

    // for _ in 0..100 {
    //     w.render();
    //     let image = w.get_image();
    // }
}
