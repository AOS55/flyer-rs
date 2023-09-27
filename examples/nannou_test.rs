use nannou::prelude::*;

struct Model {
    texture: wgpu::Texture,
}

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    // Create a new window!
    app.new_window().size(512, 512).view(view).build().unwrap();
    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path().unwrap();
    let img_path = assets.join("tiles").join("1-flower.png");
    let texture = wgpu::Texture::from_path(app, img_path).unwrap();
    Model { texture }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let win = app.window_rect();
    let r = Rect::from_w_h(100.0, 100.0).top_left_of(win);

    let draw = app.draw();
    draw.texture(&model.texture)
    .xy(Vec2::new(0.0, 0.0))
    .wh(r.wh());


    draw.to_frame(app, &frame).unwrap();

    let file_path = captured_frame_path(app, &frame);
    app.main_window().capture_frame(file_path);
}

fn captured_frame_path(app: &App, frame: &Frame) -> std::path::PathBuf {
  // Create a path that we want to save this frame to.
    app.project_path()
        .expect("failed to locate `project_path`")
        // Capture all frames to a directory called `/<path_to_nannou>/nannou/simple_capture`.
        .join(app.exe_name().unwrap())
        // Name each file after the number of the frame.
        .join(format!("{:03}", frame.nth()))
        // The extension will be PNG. We also support tiff, bmp, gif, jpeg, webp and some others.
        .with_extension("png")
}
