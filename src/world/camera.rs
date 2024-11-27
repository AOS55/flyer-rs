use nalgebra::Vector3;

pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub f: f32,
}

impl Camera {
    pub fn move_to(&mut self, position: Vector3<f64>) {
        self.x = position.x;
        self.y = position.y;
        self.z = position.z;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            f: 1.0,
        }
    }
}
