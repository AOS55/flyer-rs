use tiny_skia::*;

fn main() {
    let mut root_pixmap = Pixmap::new(480, 480).unwrap();
    let pixmap = Pixmap::load_png("assets/dynamic_objects/aircraft-north.png").unwrap();
    let pixmap_paint = PixmapPaint::default();
    let transform = Transform::identity();

    root_pixmap.draw_pixmap(240-8, 240-8, pixmap.as_ref(), &pixmap_paint, transform, None);

    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 127, 0, 200);
    paint.anti_alias = true;

    let path = {
        let mut pb = PathBuilder::new();
        const CENTER: f32 = 240.0;
        pb.move_to(CENTER, CENTER);
        pb.line_to(240.0, 240.0);
        pb.line_to(220.0, 240.0);
        pb.line_to(200.0, 240.0);
        pb.line_to(180.0, 180.0); 
        pb.finish().unwrap()
    };

    let mut stroke = Stroke::default();
    stroke.width = 2.0;
    stroke.line_cap = LineCap::Round;
    // stroke.dash = StrokeDash::new(vec![20.0, 40.0], 0.0);
    root_pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

    root_pixmap.save_png("image-test.png").unwrap();

}



