
use aerso::types::*;
use image::{DynamicImage, ImageBuffer, GenericImageView};
mod world;
mod aircraft;
mod terrain;
use world::World;

use std::env;

fn image_to_array(image: DynamicImage) -> Option<Vec<Vec<[u8; 3]>>> {

    let (width, height) = image.dimensions();
    // Create a 3D array to store the RGB values
    let mut pixel_array = vec![vec![[0; 3]; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            // Get the RGB pixel value at each coordinate
            let pixel = image.get_pixel(x, y);
            pixel_array[y as usize][x as usize] = [pixel[0], pixel[1], pixel[2]];
        }
    }

    Some(pixel_array)
}

fn main() {

    println!("current dir: {:?}", env::current_dir());

    let image = image::open("assets/tiles/1-flower.png").unwrap();
    if let Some(array) = image_to_array(image) {
        // Use the array for further processing
        println!("Array shape: [{}, {}, 3]", array.len(), array[0].len());

        // Access individual RGB values
        println!("Example RGB: [{}, {}, {}]", array[0][0][0], array[0][0][1], array[0][0][2]);
    } else {
        println!("Failed to convert image to array.");
    }


    // let mut w = World::default();
    // w.create_map(3, None, None, Some(true));
    // w.render();
    // let bytes: Vec<u8> = w.get_image();
    
    // let buffer: ImageBuffer<image::Rgb<_>, Vec<u8>> = ImageBuffer::from_raw(
    //     1024, 
    //     1024, 
    //     bytes).expect("Failed to create ImageBuffer");

    // let dynamic_image: DynamicImage = DynamicImage::ImageRgb8(buffer);
    // dynamic_image.save("sdl2_world.jpg").expect("Failed to save image");

}
