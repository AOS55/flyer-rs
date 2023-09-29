mod world;
mod aircraft;
mod terrain;
use world::World;

fn main() {

    let mut w = World::default();
    w.set_screen_dims(3840.0, 3840.0);
    w.create_map(1, None, None, Some(true));

    let pixmap = w.render();
    // println!("{:?}", pixmap.data());
    pixmap.save_png("image_big.png").unwrap();
}
