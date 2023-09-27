mod world;
mod aircraft;
mod terrain;
use world::World;

fn main() {

    let mut w = World::default();
    w.create_map(1, None, None, Some(true));

    let pixmap = w.render();
    pixmap.save_png("image.png").unwrap();

}
