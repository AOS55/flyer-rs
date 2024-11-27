use crate::vehicles::traits::VehicleState;
use aerso::density_models::ConstantDensity;
use aerso::types::*;
use aerso::wind_models::*;
use aerso::*;
use nalgebra::{UnitQuaternion, Vector3};
use std::collections::HashMap;

use crate::physics::traits::PhysicsModel;
use crate::utils::errors::SimError;
use crate::vehicles::traits::Vehicle;

use super::{
    config::AircraftConfig,
    controls::AircraftControls,
    state::AircraftState,
    systems::{Aerodynamics, PowerPlant},
};

pub struct Aircraft {
    pub name: String,
    pub state: AircraftState,
    pub controls: HashMap<String, f64>,
    pub data_path: Option<String>,
    pub aff_body: AffectedBody<Vec<f64>, f64, ConstantWind<f64>, ConstantDensity>,
}

impl Vehicle for Aircraft {
    type State = AircraftState;
    type Controls = AircraftControls;
    type Config = AircraftConfig;

    fn new(config: Self::Config) -> Result<Self, SimError> {
        // Initialize aerodynamics and powerplant from config
        let aero = Aerodynamics::from_json(&config.name, None)?;
        let power = PowerPlant::pt6();

        // Create initial kinematic body
        let k_body = Body::new(
            config.mass,
            config.inertia,
            Vector3::zeros(),
            Vector3::zeros(),
            UnitQuaternion::identity(),
            Vector3::zeros(),
        );

        let a_body = AeroBody::new(k_body);

        let aff_body = AffectedBody {
            body: a_body,
            effectors: vec![Box::new(aero), Box::new(power)],
        };

        // Initialize default controls
        let controls = HashMap::from([
            ("aileron".to_string(), 0.0),
            ("elevator".to_string(), 0.0),
            ("tla".to_string(), 0.0),
            ("rudder".to_string(), 0.0),
        ]);

        let state = AircraftState {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            rates: Vector3::zeros(),
            air_speed: 0.0,
            ground_speed: 0.0,
            altitude: 0.0,
            heading: 0.0,
            flight_path_angle: 0.0,
        };

        Ok(Self {
            name: config.name,
            state,
            controls,
            data_path: None,
            aff_body,
        })
    }

    fn set_controls(&mut self, controls: Self::Controls) {
        self.controls
            .insert("aileron".to_string(), controls.aileron);
        self.controls
            .insert("elevator".to_string(), controls.elevator);
        self.controls.insert("tla".to_string(), controls.throttle);
        self.controls.insert("rudder".to_string(), controls.rudder);
    }

    fn update_state(&mut self, physics: &dyn PhysicsModel) {
        // Extract current controls in correct order
        let control_keys = vec!["aileron", "elevator", "tla", "rudder"];
        let mut control_in: Vec<f64> = Vec::new();

        for key in control_keys {
            control_in.push(self.controls[key]);
        }

        // Step the physics
        self.aff_body.step(physics.get_timestep(), &control_in);

        // Update our state
        self.state.position = self.aff_body.position();
        self.state.velocity = self.aff_body.velocity();
        self.state.attitude = self.aff_body.attitude();
        self.state.rates = self.aff_body.rates();

        let airstate = self.aff_body.get_airstate();
        self.state.air_speed = airstate.airspeed;
        self.state.ground_speed = self.state.velocity.norm();
        self.state.altitude = -self.state.position.z;

        let (_, pitch, yaw) = self.state.attitude.euler_angles();
        self.state.heading = yaw;
        self.state.flight_path_angle = pitch;
    }

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn reset(&mut self, state: Self::State) {
        self.state = state;

        // Reset aerso physics state
        self.aff_body.set_state(StateVector::from_vec(vec![
            state.position[0],
            state.position[1],
            state.position[2],
            state.velocity[0],
            state.velocity[1],
            state.velocity[2],
            state.attitude[0],
            state.attitude[1],
            state.attitude[2],
            state.attitude[3],
            state.rates[0],
            state.rates[1],
            state.rates[2],
        ]));

        // Reset controls
        for (_, value) in self.controls.iter_mut() {
            *value = 0.0;
        }
    }
}

impl Aircraft {
    pub fn new_with_state(
        aircraft_name: &str,
        initial_position: Vector3<f64>,
        initial_velocity: Vector3<f64>,
        initial_attitude: UnitQuaternion<f64>,
        initial_rates: Vector3<f64>,
        controls: Option<HashMap<String, f64>>,
        data_path: Option<String>,
    ) -> Result<Self, SimError> {
        let aero = Aerodynamics::from_json(aircraft_name, data_path.as_deref())?;
        let power = PowerPlant::pt6();

        let k_body = Body::new(
            aero.mass,
            aero.inertia,
            initial_position,
            initial_velocity,
            initial_attitude,
            initial_rates,
        );

        let a_body = AeroBody::new(k_body);

        let aff_body = AffectedBody {
            body: a_body,
            effectors: vec![Box::new(aero), Box::new(power)],
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

        let state = AircraftState {
            position: initial_position,
            velocity: initial_velocity,
            attitude: initial_attitude,
            rates: initial_rates,
            air_speed: initial_velocity.norm(),
            ground_speed: initial_velocity.norm(),
            altitude: -initial_position.z,
            heading: initial_attitude.euler_angles().2,
            flight_path_angle: initial_attitude.euler_angles().1,
        };

        Ok(Self {
            name: aircraft_name.to_string(),
            state,
            controls,
            data_path,
            aff_body,
        })
    }

    pub fn clone_with_state(
        &self,
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        attitude: UnitQuaternion<f64>,
        rates: Vector3<f64>,
    ) -> Result<Self, SimError> {
        Aircraft::new_with_state(
            &self.name,
            position,
            velocity,
            attitude,
            rates,
            Some(self.controls.clone()),
            self.data_path.clone(),
        )
    }

    pub fn step(&mut self, dt: f64) {
        // Extract controls in correct order
        let control_keys = vec!["aileron", "elevator", "tla", "rudder"];
        let mut control_in: Vec<f64> = Vec::new();

        for key in control_keys {
            control_in.push(self.controls[key]);
        }

        // Step physics
        self.aff_body.step(dt, &control_in);

        // Update state
        self.state.position = self.aff_body.position();
        self.state.velocity = self.aff_body.velocity();
        self.state.attitude = self.aff_body.attitude();
        self.state.rates = self.aff_body.rates();

        let airstate = self.aff_body.get_airstate();
        self.state.air_speed = airstate.airspeed;
        self.state.ground_speed = self.state.velocity.norm();
        self.state.altitude = -self.state.position.z;

        let (_, pitch, yaw) = self.state.attitude.euler_angles();
        self.state.heading = yaw;
        self.state.flight_path_angle = pitch;
    }

    pub fn set_data_path(&mut self, data_path: String) {
        self.data_path = Some(data_path);
    }

    fn get_control_vector(&self) -> Vec<f64> {
        vec![
            self.controls["aileron"],
            self.controls["elevator"],
            self.controls["tla"],
            self.controls["rudder"],
        ]
    }
}

impl Clone for Aircraft {
    fn clone(&self) -> Self {
        let name = self.name.clone();
        let pos = self.state.position;
        let vel = self.state.velocity;
        let att = self.state.attitude;
        let rates = self.state.rates;
        let controls = self.controls.clone();
        let data_path = self.data_path.clone();

        Aircraft::new_with_state(&name, pos, vel, att, rates, Some(controls), data_path).unwrap()
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
