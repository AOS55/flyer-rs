use image::DynamicImage;

pub fn image_to_array(image: DynamicImage) -> Vec<Vec<[u8; 3]>> {

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

    pixel_array
}