extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use sdl2::render::Canvas;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

use image::{DynamicImage, ImageBuffer};

pub fn main() -> () {

    let masks = PixelFormatEnum::RGB24.into_masks().unwrap();
    let surface = Surface::from_pixelmasks(1024, 1024, masks).unwrap();
    
    let mut canvas = surface.into_canvas().unwrap();
    canvas.set_draw_color(Color::RGB(255, 210, 0));
    canvas.fill_rect(Rect::new(10, 10, 780, 580)).unwrap();
    // canvas.clear();
    // canvas.present();
    
    let rect = Rect::new(0, 0, 1024, 1024);
    let bytes = canvas.read_pixels(rect, PixelFormatEnum::RGB24).unwrap();
    // println!("{:?}", pixels);

    let buffer: ImageBuffer<image::Rgb<_>, Vec<u8>> = ImageBuffer::from_raw(
        1024, 
        1024, 
        bytes).expect("Failed to create ImageBuffer");

    let dynamic_image: DynamicImage = DynamicImage::ImageRgb8(buffer);
    dynamic_image.save("sdl2_image.jpg").expect("Failed to save image");

}
