// extern crate sdl2;

// use sdl2::image::{LoadTexture, LoadSurface};
// use sdl2::pixels::PixelFormatEnum;
// use sdl2::surface::Surface;
// use sdl2::render::Canvas;
// use sdl2::rect::Rect;
// use sdl2::pixels::Color;
// use sdl2::rwops::RWops;

// use image::io::Reader;
// use image::{DynamicImage, ImageBuffer};

// extern crate flyer;
// use flyer::Terrain;

// use std::fs;
// use std::collections::HashMap;

// // use crate::terrain::Terrain;

// pub fn main() -> () {

//     let tile_dir: Vec<_> = match fs::read_dir("resources/tiles") {
//         Ok(td) => {
//             td
//             .filter_map(|entry| Some(entry.ok()?.path()))
//             .collect()
//         },
//         Err(_td) => {
//             eprintln!("{}", "Tiles dir not found in context");
//             std::process::exit(1);
//         }
//     };

//     let mut asset_map: HashMap<String, Surface<'_>> = HashMap::new();
//         for path in tile_dir {
//             let path_str = path.to_str().unwrap_or_default().to_string();
//             match Surface::from_file(&path) {
//                 Ok(surf) => {
//                     let name: Vec<&str> = path_str.split('/').collect();
//                     let name = name[2].to_string();
//                     let name: Vec<&str> = name.split('.').collect();
//                     let name = name[0].to_string();
//                     // println!("name: {}", name);
//                     asset_map.insert(name, surf);
//                 }
//                 Err(err) => {
//                     println!("Path is: {}", path_str);
//                     eprintln!("Error {}", err);
//                 }
//             }
//         }

//     let grass = &asset_map["1-flower"];

//     let masks = PixelFormatEnum::RGB24.into_masks().unwrap();
//     let surface = Surface::from_pixelmasks(1024, 1024, masks).unwrap();
    
//     let mut canvas: Canvas<Surface<'_>> = surface.into_canvas().unwrap();
//     canvas.clear();

//     let texture_creator = canvas.texture_creator();
//     let texture = texture_creator.create_texture_from_surface(&grass).unwrap();
//     let rect = Rect::new(10, 10, 128, 128);
//     canvas.copy(&texture, None, rect).unwrap();
//     canvas.present();

//     // canvas.set_draw_color(Color::RGB(255, 210, 0));
//     // canvas.fill_rect(Rect::new(10, 10, 780, 580)).unwrap();
//     // canvas.clear();
    
//     let rect = Rect::new(-1000, -1000, 1024, 1024);
//     let bytes = canvas.read_pixels(rect, PixelFormatEnum::RGB24).unwrap();
//     // println!("{:?}", pixels);

//     let buffer: ImageBuffer<image::Rgb<_>, Vec<u8>> = ImageBuffer::from_raw(
//         1024, 
//         1024, 
//         bytes).expect("Failed to create ImageBuffer");

//     let dynamic_image: DynamicImage = DynamicImage::ImageRgb8(buffer);
//     dynamic_image.save("sdl2_image.jpg").expect("Failed to save image");

// }
