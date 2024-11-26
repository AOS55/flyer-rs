use crate::aircraft::Aircraft;
use crate::utils::AircraftError;
use aerso::types::*;
use std::{env, path::PathBuf};

extern crate nalgebra as na;
use argmin::core::{CostFunction, Error};
use na::DVector;

#[derive(Clone, Copy)]
pub struct Trim {
    /// Altitude to maintain
    pub alt: f64,
    /// Airspeed to maintain
    pub airspeed: f64,
}

impl Trim {
    const FPS: u32 = 100;
    const EXP_LEN: f32 = 200.0;

    pub fn eval(self, u: &Vec<f64>) -> Result<f64, AircraftError> {
        let dt = 1.0 / Self::FPS as f64;

        // This allows the trim to run in test suite without placing data files at root dir
        let f_path = if env::current_dir()?.file_name().unwrap() == PathBuf::from("flyer-env") {
            Some(String::from("flyer_env/envs/data/"))
        } else {
            None
        };

        let mut aircraft = Aircraft::new(
            "TO",
            Vector3::new(0.0, 0.0, self.alt),
            Vector3::new(self.airspeed, 0.0, 0.0),
            UnitQuaternion::from_euler_angles(0.0, u[0], 0.0),
            Vector3::zeros(),
            None,
            f_path,
        )?;

        let controls = vec![0.0, u[1], u[2], 0.0];
        let mut total_cost = 0.0;
        let mut time = 0.0;

        for _ in 0..(Self::FPS * (Self::EXP_LEN as u32)) {
            aircraft.aff_body.step(dt, &controls);

            if time > 0.1 {
                let current_cost = (aircraft.velocity()[0] - self.airspeed).powf(2.0)
                    + (aircraft.velocity()[2]).powf(2.0);
                total_cost += current_cost * dt;
            };
            time += dt;
        }

        Ok(total_cost)
    }
}

impl CostFunction for Trim {
    type Param = DVector<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        self.eval(param.data.as_vec())
            .map_err(|e| Error::msg(e.to_string()))
    }
}
