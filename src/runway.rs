use glam::Vec2;
use std::collections::HashMap;
use std::f32::consts::PI;

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

    pub fn approach_points(&self) -> HashMap<String, Vec<f32>> {
        
        let touchdown_fraction = 0.8;  // fraction of runway that is landable, used to determine touchdown point
        let faf_dist = 3000.0;  // distance from final approach fix to touchdown_point [m]
        let inaf_dist = 2000.0; // distance from intermediate fix to touchdown point [m]
        let iaf_dist = 2000.0; // distance from initial approach fix to the intermediate approach fix
        let intercept_angle = 30.0 * PI/180.0;  //

        let faf_rot_vec = Vec2::from_angle(self.heading.to_radians() - PI);
        let touchdown_point = faf_rot_vec.rotate(Vec2::new(self.dims[1] * (touchdown_fraction/2.0), 0.0)) + self.pos;

        let faf = faf_rot_vec.rotate(Vec2::new(faf_dist, 0.0)) + touchdown_point;
        
        let inaf_l_rot_vec = Vec2::from_angle(self.heading.to_radians() - PI + intercept_angle);
        let inaf_l = inaf_l_rot_vec.rotate(Vec2::new(inaf_dist, 0.0)) + faf;

        let inaf_r_rot_vec = Vec2::from_angle(self.heading.to_radians() - PI - intercept_angle);
        let inaf_r = inaf_r_rot_vec.rotate(Vec2::new(inaf_dist, 0.0)) + faf;

        let iaf_l_rot_vec = Vec2::from_angle(self.heading.to_radians() - PI/2.0);
        let iaf_l = iaf_l_rot_vec.rotate(Vec2::new(iaf_dist, 0.0)) + inaf_l;

        let iaf_r_rot_vec = Vec2::from_angle(self.heading.to_radians() + PI/2.0);
        let iaf_r = iaf_r_rot_vec.rotate(Vec2::new(iaf_dist, 0.0)) + inaf_r;

        let points: HashMap<String, Vec<f32>> = HashMap::from([
            ("touchdown".to_string(), vec![touchdown_point[0], touchdown_point[1]]),
            ("faf".to_string(), vec![faf[0], faf[1]]),
            ("inaf_l".to_string(), vec![inaf_l[0], inaf_l[1]]),
            ("inaf_r".to_string(), vec![inaf_r[0], inaf_r[1]]),
            ("iaf_l".to_string(), vec![iaf_l[0], iaf_l[1]]),
            ("iaf_r".to_string(), vec![iaf_r[0], iaf_r[1]])
        ]);

        points

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
