mod world;
mod aircraft;
mod terrain;
mod runway;
use world::World;

use glam::Vec2;

fn main() {

    let mut w = World::default();
    w.set_screen_dims(480.0, 480.0);
    w.create_map(1, Some(vec![256, 256]), None, Some(true));
    w.create_runway();
    w.camera.move_camera(vec![0.0, 0.0, -1000.0]);
    // w.runway.unwrap().on_runway(Vec2::new(-499.0, -10.0));
    let pixmap = w.render();
    // // println!("{:?}", pixmap.data());
    pixmap.save_png("image-test.png").unwrap();
}
