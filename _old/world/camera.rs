pub struct Camera {
    pub x: f64, // camera's x-position
    pub y: f64, // camera's y-position
    pub z: f64, // camera's z-position
    pub f: f64, // camera's reconstruction ratio/zoom
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 5000.0,
            f: 1.0,
        }
    }
}

impl Camera {
    #[allow(dead_code)]
    fn new(x: f64, y: f64, z: f64, f: Option<f64>) -> Self {
        let f = if let Some(focal_dist) = f {
            focal_dist
        } else {
            1.0
        };

        Self { x, y, z, f }
    }

    pub fn move_camera(&mut self, pos: Vec<f64>) {
        self.x = pos[0];
        self.y = pos[1];
        self.z = -1.0 * pos[2];
    }
}
