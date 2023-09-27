use tiny_skia::*;

fn main() {
    let mut root_pixmap = Pixmap::new(200, 200).unwrap();
    let pixmap = Pixmap::load_png("assets/tiles/1-flower.png").unwrap();
    let paint = PixmapPaint::default();
    let transform = Transform::identity(); 
    root_pixmap.draw_pixmap(10, 10, pixmap.as_ref(), &paint, transform, None);
    root_pixmap.draw_pixmap(40, 40, pixmap.as_ref(), &paint, transform, None);

    let alt_transform = Transform::from_row(1.0, 0.0, 0.0, 1.0, 30.0, 30.0);
    root_pixmap.draw_pixmap(10, 10, pixmap.as_ref(), &paint, alt_transform, None);
    root_pixmap.save_png("image.png").unwrap();
}