use super::config::{AtmosphereModel, WindModelConfig};
use crate::physics::aerso::config::AersoConfig;
use crate::physics::traits::{AerodynamicsModel, PhysicsModel};
use crate::utils::errors::SimError;
use crate::vehicles::aircraft::state::AircraftState;
use crate::vehicles::traits::VehicleState;

use aerso::density_models::{ConstantDensity, StandardDensity};
use aerso::types::Force;
use aerso::wind_models::{ConstantWind, LogWind, PowerWind};
use aerso::{AeroBody, AffectedBody, AirState, Body, DensityModel, WindModel};
use nalgebra::{UnitQuaternion, Vector3};

pub struct AersoPhysics {
    // Wrapped aerso components
    aero_body: AffectedBody<Vec<f64>, f64, Box<dyn WindModel<f64>>, Box<dyn DensityModel<f64>>>,
    // Cache of computed forces and accelerations
    cached_forces: Vec<Force<f64>>,
    cached_accelerations: Vector3<f64>,
    // Configuration
    config: AersoConfig,
}

impl PhysicsModel for AersoPhysics {
    type State = AersoState;
    type Parameters = AersoConfig;

    fn new(params: Self::Parameters) -> Result<Self, SimError> {
        // Create the aerso body and components from config
        let aero = params.create_aerodynamics()?;
        let power = params.create_powerplant()?;

        let body = Body::new(
            params.mass,
            params.inertia,
            Vector3::zeros(),
            Vector3::zeros(),
            UnitQuaternion::identity(),
            Vector3::zeros(),
        );

        let wind_model = Box::new(match params.wind_model {
            WindModelConfig::Constant(wind) => ConstantWind::new(wind),
            WindModelConfig::LogWind {
                d,
                z0,
                u_star,
                bearing,
            } => Box::new(LogWind::new(d, z0, u_star, bearing)),
            WindModelConfig::PowerWind {
                u_r,
                z_r,
                bearing,
                alpha,
            } => Box::new(PowerWind::new_with_alpha(u_r, z_r, bearing, alpha)),
        });

        let density_model: Box<dyn DensityModel<f64>> = match params.atmosphere_model {
            AtmosphereModel::Constant => Box::new(ConstantDensity),
            AtmosphereModel::Standard => Box::new(StandardDensity),
        };

        let aero_body = AffectedBody {
            body: AeroBody::new(body),
            effectors: vec![Box::new(aero), Box::new(power)],
            wind_model,
            density_model,
        };

        Ok(Self {
            aero_body,
            cached_forces: Vec::new(),
            cached_accelerations: Vector3::zeros(),
            config: params,
        })
    }

    fn step(&mut self, state: &mut dyn VehicleState, dt: f64) {
        // Convert vehicle state to aerso state
        let aerso_state: AersoState = state.into();

        // Update aerso body state
        self.aero_body.set_state(aerso_state.into());

        // Create control vector for aerso
        let controls = self.get_control_vector();

        // Step the aerso simulation
        self.aero_body.step(dt, &controls);

        // Cache forces and accelerations
        self.cached_forces = self.collect_forces();
        self.cached_accelerations = self.aero_body.acceleration();

        // Update vehicle state from aerso
        self.update_vehicle_state(state);
    }

    fn reset(&mut self) {
        self.cached_forces.clear();
        self.cached_accelerations = Vector3::zeros();
    }

    fn get_forces(&self) -> Vec<Force<f64>> {
        self.cached_forces.clone()
    }

    fn get_accelerations(&self) -> Vector3<f64> {
        self.cached_accelerations
    }
}

impl AerodynamicsModel for AersoPhysics {
    fn get_aero_forces(&self, state: &dyn VehicleState) -> Vec<Force<f64>> {
        // Implement the method to get aerodynamic forces
        self.cached_forces.clone()
    }

    fn get_air_data(&self, state: &dyn VehicleState) -> AirState {
        // Implement the method to get air data
        self.aero_body.get_airstate()
    }
}

impl AersoPhysics {
    // Helper methods for state conversion
    fn convert_to_aerso_state(&self, state: &dyn VehicleState) -> AersoState {
        AersoState {
            position: state.position(),
            velocity: state.velocity(),
            attitude: state.attitude(),
            rates: state.rates(),
        }
    }

    fn update_vehicle_state(&self, state: &mut dyn VehicleState) {
        if let Some(aircraft_state) = state.downcast_mut::<AircraftState>() {
            aircraft_state.position = self.aero_body.position();
            aircraft_state.velocity = self.aero_body.velocity();
            aircraft_state.attitude = self.aero_body.attitude();
            aircraft_state.rates = self.aero_body.rates();

            // Update derived state
            let airstate = self.aero_body.get_airstate();
            aircraft_state.air_speed = airstate.airspeed;
            // Update other derived states...
        }
    }

    fn collect_forces(&self) -> Vec<Force<f64>> {
        // Collect all forces from aerso effectors
        let mut forces = Vec::new();
        // Implement force collection from aerso
        forces
    }

    fn get_control_vector(&self) -> Vec<f64> {
        // Convert control state to aerso format
        vec![0.0; 4] // Placeholder
    }
}

// State representation for aerso physics
#[derive(Debug, Clone)]
pub struct AersoState {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub attitude: UnitQuaternion<f64>,
    pub rates: Vector3<f64>,
}
