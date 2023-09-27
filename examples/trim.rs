extern crate flyer;
use flyer::Aircraft;

use aerso::types::*;

use std::f64::consts::PI;

extern crate nalgebra as na;

use argmin::core::{CostFunction, Error, Executor};
use argmin::core::observers::{ObserverMode, SlogLogger};
use argmin::solver::particleswarm::ParticleSwarm;
use nalgebra::{dvector, DVector};

#[derive(Clone, Copy)]
struct Trim {
    /// Altitude to maintain
    alt: f64,
    /// Airspeed to maintain
    airspeed: f64
}

impl Trim {
    /// Evaluate the cost of trimming when given the input params [pitch, elevator, tla]

    const FPS: u32 = 100;
    const EXP_LEN: f32 = 200.0;

    fn eval(self, u: &Vec<f64>) -> f64 {
        
        let dt = 1.0/Self::FPS as f64;

        let mut aircraft = Aircraft::new(
            "TO",
            Vector3::new(0.0, 0.0, self.alt),
            Vector3::new(self.airspeed, 0.0, 0.0),
            UnitQuaternion::from_euler_angles(0.0, u[0], 0.0),
            Vector3::zeros()
        );

        let controls = vec![u[1], 0.0, u[2], 0.0];
        let mut total_cost = 0.0;
        let mut time = 0.0;
        
        for _ in 0..(Self::FPS * (Self::EXP_LEN as u32)) {
            aircraft.aff_body.step(dt, &controls);

            if time > 0.1 {
                let current_cost = (aircraft.velocity()[0] - self.airspeed).powf(2.0) + (aircraft.velocity()[2]).powf(2.0);
                total_cost += current_cost * dt;
            };
            time += dt;
        };
        // println!("u: {:?}, total_cost: {}", u, total_cost);
        // println!("u-airspeed: {}, w: {}", aircraft.velocity()[0] - self.airspeed, aircraft.velocity()[2]);
        total_cost
            
    }
}

impl CostFunction for Trim {
    type Param = DVector<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        Ok(self.eval(param.data.as_vec()))
    }
}

fn run() -> Result<(), Error> {

    let cost = Trim {
        alt: 1000.0,
        airspeed: 100.0
    };

    {   
        let solver = ParticleSwarm::new((dvector![-20.0 * (PI/180.0), -4.0 * (PI/180.0), 0.0], dvector![20.0 * (PI/180.0), 4.0 * (PI/180.0), 1.0]), 40);
        
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100))
            .add_observer(SlogLogger::term(), ObserverMode::Always)
            .run()?;

        // Print Result
        println!("Result: {}", res)
    }

    Ok(())
}

fn main() {
    if let Err(ref e) = run() {
        println!("{}", e);
        std::process::exit(1);
    }
}
