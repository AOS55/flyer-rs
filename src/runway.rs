use glam::Vec2;

pub struct Runway {
    pub name: String,
    pub asset: String,
    pub pos: Vec2,  // center point of runway [m]
    pub dims: Vec2,  // dimensions of runway [m]
    pub heading: f32 // heading of runway [m]
}

impl Default for Runway {
    
    fn default() -> Self {
        Self{
            name: "runway".to_string(),
            asset: "runway".to_string(),
            pos: Vec2::new(0.0, 0.0),
            dims: Vec2::new(25.0, 1000.0),
            heading: 0.0
        }
    }
}

impl Runway {

    pub fn on_runway(&self, test_point: Vec2) -> bool {
        
        let rot_vec = Vec2::from_angle(self.heading.to_radians());
        let top_right = rot_vec.rotate(Vec2::new(self.dims[1] / 2.0, self.dims[0]/2.0)) + self.pos;
        let bottom_right = rot_vec.rotate(Vec2::new(self.dims[1] / 2.0, -self.dims[0]/2.0)) + self.pos;
        let top_left = rot_vec.rotate(Vec2::new(-self.dims[1] / 2.0, self.dims[0]/2.0)) + self.pos;
        let bottom_left = rot_vec.rotate(Vec2::new(-self.dims[1] / 2.0, -self.dims[0]/2.0)) + self.pos;
        
        let polygon = vec![
            top_right,
            bottom_right,
            bottom_left,
            top_left
        ];
        
        is_point_inside_polygon(test_point, polygon)

    }

}

fn is_point_inside_polygon(point: Vec2, polygon_points: Vec<Vec2>) -> bool {
    let n = polygon_points.len();
    let mut inside = false;
    let mut idy: usize = n - 1;

    for idx in 0..n {
        if (polygon_points[idx][1] < point[1] && polygon_points[idy][1] >= point[1])
            || (polygon_points[idy][1] < point[1] && polygon_points[idx][1] >= point[1])
        {
            if polygon_points[idx][0]
                + (point.y - polygon_points[idx][1])
                    / (polygon_points[idy][1] - polygon_points[idx][1])
                    * (polygon_points[idy][0] - polygon_points[idx][0])
                < point[0]
            {
                inside = !inside;
            }
        }
        idy = idx;
    }

    inside
}
