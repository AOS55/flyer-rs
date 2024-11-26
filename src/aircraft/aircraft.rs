#![warn(clippy::all)]

use crate::aircraft::{Aerodynamics, PowerPlant};
use crate::utils::AircraftError;
use aerso::density_models::ConstantDensity;
use aerso::types::*;
use aerso::wind_models::*;
use aerso::*;
use std::collections::HashMap;

/// Represent a fixed-wing aircraft
pub struct Aircraft {
    // Name of the aircraft
    pub name: String,
    // Effected aircraft body
    pub aff_body: AffectedBody<Vec<f64>, f64, ConstantWind<f64>, ConstantDensity>,
    // Aircraft controls
    pub controls: HashMap<String, f64>,
    // Path to the aircraft json directory
    pub data_path: Option<String>,
}

impl Aircraft {
    #[inline]
    pub fn new(
        aircraft_name: &str,
        initial_position: Vector3<f64>,
        initial_velocity: Vector3<f64>,
        initial_attitude: UnitQuaternion<f64>,
        initial_rates: Vector3<f64>,
        controls: Option<HashMap<String, f64>>,
        data_path: Option<String>,
    ) -> Result<Self, AircraftError> {
        let path = data_path.as_deref();

        let aero = Aerodynamics::from_json(aircraft_name, path);
        let power = PowerPlant::pt6();

        let k_body = Body::new(
            aero.as_ref().unwrap().mass,
            aero.as_ref().unwrap().inertia,
            initial_position,
            initial_velocity,
            initial_attitude,
            initial_rates,
        );

        let a_body = AeroBody::new(k_body);

        let aff_body = AffectedBody {
            body: a_body,
            effectors: vec![Box::new(aero.unwrap()), Box::new(power)],
        };

        let controls = match controls {
            Some(controls) => controls,
            None => HashMap::from([
                ("aileron".to_string(), 0.0),
                ("elevator".to_string(), 0.0),
                ("tla".to_string(), 0.0),
                ("rudder".to_string(), 0.0),
            ]),
        };

        Ok(Self {
            name: aircraft_name.to_string(),
            aff_body,
            controls,
            data_path,
        })
    }

    /// Set the controls
    /// # Arguments
    /// * `controls` - HashMap usually containing ["aileron", "elevator", "tla", "rudder"]
    #[allow(dead_code)]
    pub fn act(&mut self, controls: HashMap<String, f64>) {
        self.controls = controls;
    }

    /// Step the simulation
    #[allow(dead_code)]
    pub fn step(&mut self, dt: f64) {
        // HashMaps aren't ordered so we need to make sure everything comes out in the correct sequence
        let control_keys = vec!["aileron", "elevator", "tla", "rudder"];
        let mut control_in: Vec<f64> = Vec::new();

        for key in control_keys {
            control_in.push(self.controls[key]);
        }

        // let controls: Vec<_> = self.controls.values().cloned().collect();
        self.aff_body.step(dt, &control_in);
    }
}

impl Clone for Aircraft {
    fn clone(&self) -> Self {
        let name: String = self.name.clone();
        let pos = self.position();
        let vel = self.velocity();
        let att = self.attitude();
        let rates = self.rates();
        let controls = self.controls.clone();
        let data_path = self.data_path.clone();
        let ac = Aircraft::new(&name, pos, vel, att, rates, Some(controls), data_path).unwrap();

        Self {
            name: ac.name,
            aff_body: ac.aff_body,
            controls: ac.controls,
            data_path: ac.data_path,
        }
    }
}

impl StateView for Aircraft {
    fn position(&self) -> Vector3<f64> {
        self.aff_body.position()
    }

    fn velocity_in_frame(&self, frame: Frame) -> Vector3<f64> {
        self.aff_body.velocity_in_frame(frame)
    }

    fn attitude(&self) -> UnitQuaternion<f64> {
        self.aff_body.attitude()
    }

    fn rates_in_frame(&self, frame: Frame) -> Vector3<f64> {
        self.aff_body.rates_in_frame(frame)
    }

    fn statevector(&self) -> StateVector<f64> {
        self.aff_body.statevector()
    }
}
