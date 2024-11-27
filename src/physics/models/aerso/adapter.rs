use super::config::{AersoConfig, AtmosphereModel, WindModelConfig};
use super::state::AersoPhysicsState;
use crate::physics::components::{Force, ForceSystem, ForceType, Moment, ReferenceFrame};
use crate::physics::{PhysicsError, PhysicsModel};
use crate::state::SpatialOperations;

use aerso::density_models::{ConstantDensity, StandardDensity};
use aerso::wind_models::{ConstantWind, LogWind, PowerWind};
use aerso::{AeroBody, AffectedBody, Body, DensityModel, WindModel};
use nalgebra::{UnitQuaternion, Vector3};

// Create wrapper types for aerso forces/torques
pub struct AersoForceWrapper(aerso::types::Force<f64>);

pub struct AersoTorqueWrapper(aerso::types::Torque<f64>);

// Implement conversion from wrapper to Vector3
impl From<AersoForceWrapper> for Vector3<f64> {
    fn from(force: AersoForceWrapper) -> Self {
        force.0.into_body_frame()
    }
}

impl From<AersoTorqueWrapper> for Vector3<f64> {
    fn from(torque: AersoTorqueWrapper) -> Self {
        torque.0.into_body_frame()
    }
}

// Implement conversion from aerso types to our wrappers
impl From<aerso::types::Force<f64>> for AersoForceWrapper {
    fn from(force: aerso::types::Force<f64>) -> Self {
        AersoForceWrapper(force)
    }
}

impl From<aerso::types::Torque<f64>> for AersoTorqueWrapper {
    fn from(torque: aerso::types::Torque<f64>) -> Self {
        AersoTorqueWrapper(torque)
    }
}

pub struct AersoPhysics {
    // Core aerso components
    aerso_body: AffectedBody<Vec<f64>, f64, Box<dyn WindModel<f64>>, Box<dyn DensityModel<f64>>>,
    force_system: ForceSystem,
    config: AersoConfig,
}

impl AersoPhysics {
    fn create_wind_model(config: &WindModelConfig) -> Box<dyn WindModel<f64>> {
        match config {
            WindModelConfig::Constant(wind) => Box::new(ConstantWind::new(*wind)),
            WindModelConfig::LogWind {
                d,
                z0,
                u_star,
                bearing,
            } => Box::new(LogWind::new(*d, *z0, *u_star, *bearing)),
            WindModelConfig::PowerWind {
                u_r,
                z_r,
                bearing,
                alpha,
            } => Box::new(PowerWind::new_with_alpha(*u_r, *z_r, *bearing, *alpha)),
        }
    }

    fn create_density_model(model: &AtmosphereModel) -> Box<dyn DensityModel<f64>> {
        match model {
            AtmosphereModel::Constant => Box::new(ConstantDensity),
            AtmosphereModel::Standard => Box::new(StandardDensity),
        }
    }

    fn create_initial_body(config: &AersoConfig) -> Body<f64> {
        Body::new(
            config.mass,
            config.inertia.clone(),
            Vector3::zeros(),
            Vector3::zeros(),
            UnitQuaternion::identity(),
            Vector3::zeros(),
        )
    }

    fn update_aerso_state(&mut self, state: &AersoPhysicsState) -> Result<(), PhysicsError> {
        let spatial = &state.spatial;
        let state_vec = vec![
            spatial.position.x,
            spatial.position.y,
            spatial.position.z,
            spatial.velocity.x,
            spatial.velocity.y,
            spatial.velocity.z,
            spatial.attitude.w,
            spatial.attitude.i,
            spatial.attitude.j,
            spatial.attitude.k,
            spatial.angular_velocity.x,
            spatial.angular_velocity.y,
            spatial.angular_velocity.z,
        ];

        self.aerso_body
            .set_state(aerso::types::StateVector::from_vec(state_vec));
        Ok(())
    }

    fn compute_aerodynamic_forces(
        &mut self,
        state: &AersoPhysicsState,
    ) -> Result<(), PhysicsError> {
        // Get current air state from aerso
        let air_state = self.aerso_body.get_airstate();

        // Compute forces using aerso's effectors
        let controls = self.get_control_vector(state);
        for effector in &self.aerso_body.effectors {
            let (force, moment) =
                effector.get_effect(air_state, state.spatial.angular_velocity, &controls);

            // Convert using wrappers
            let force_vec: Vector3<f64> = AersoForceWrapper::from(force).into();
            let moment_vec: Vector3<f64> = AersoTorqueWrapper::from(moment).into();

            // Add forces
            self.force_system.add_force(Force::new(
                force_vec,
                None, // application point
                ReferenceFrame::Body,
                ForceType::Aerodynamic,
                format!("aero_force_{}", effector.name()), // unique identifier
            ));

            // Add moments
            self.force_system.add_moment(Moment::new(
                moment_vec,
                ReferenceFrame::Body,
                ForceType::Aerodynamic,
                format!("aero_moment_{}", effector.name()),
            ));
        }

        Ok(())
    }

    fn get_control_vector(&self, state: &AersoPhysicsState) -> Vec<f64> {
        // Convert control inputs to aerso format
        // This would need to be implemented based on your control system
        vec![0.0; 4] // Example default
    }

    fn integrate_state(
        &mut self,
        state: &mut AersoPhysicsState,
        dt: f64,
    ) -> Result<(), PhysicsError> {
        // Step the aerso simulation
        self.aerso_body.step(dt, &self.get_control_vector(state));

        // Update our state from aerso
        let aerso_state = self.aerso_body.statevector();
        let position = Vector3::new(aerso_state[0], aerso_state[1], aerso_state[2]);
        let velocity = Vector3::new(aerso_state[3], aerso_state[4], aerso_state[5]);
        let attitude = UnitQuaternion::new_unchecked(nalgebra::Vector4::new(
            aerso_state[6],
            aerso_state[7],
            aerso_state[8],
            aerso_state[9],
        ));
        let angular_velocity = Vector3::new(aerso_state[10], aerso_state[11], aerso_state[12]);

        state.set_position(position)?;
        state.set_velocity(velocity)?;
        state.set_attitude(attitude)?;
        state.set_angular_velocity(angular_velocity)?;

        Ok(())
    }
}

impl PhysicsModel for AersoPhysics {
    type State = AersoPhysicsState;
    type Config = AersoConfig;

    fn new(config: Self::Config) -> Result<Self, PhysicsError> {
        let wind_model = Self::create_wind_model(&config.wind_model);
        let density_model = Self::create_density_model(&config.atmosphere_model);
        let initial_body = Self::create_initial_body(&config);

        let aero_body = AeroBody::new_with_models(initial_body, wind_model, density_model);
        let affected_body = AffectedBody {
            body: aero_body,
            effectors: vec![], // Add effectors based on config
        };

        Ok(Self {
            aerso_body: affected_body,
            force_system: ForceSystem::new(),
            config,
        })
    }

    fn step(&mut self, state: &mut Self::State, dt: f64) -> Result<(), PhysicsError> {
        // Clear previous step's forces
        self.force_system.clear();

        // Update aerso's internal state
        self.update_aerso_state(state)?;

        // Compute new forces
        self.compute_aerodynamic_forces(state)?;

        // Integrate state forward
        self.integrate_state(state, dt)?;

        Ok(())
    }

    fn reset(&mut self) {
        self.force_system.clear();
        // Reset aerso body to initial conditions
        let initial_body = Self::create_initial_body(&self.config);
        self.aerso_body.set_state(initial_body.statevector());
    }

    fn get_force_system(&self) -> &ForceSystem {
        &self.force_system
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aerso_physics_creation() {
        let config = AersoConfig::default(); // You'll need to implement this
        let physics = AersoPhysics::new(config).unwrap();
        assert!(physics.force_system.is_empty());
    }

    #[test]
    fn test_physics_step() {
        let config = AersoConfig::default();
        let mut physics = AersoPhysics::new(config).unwrap();
        let mut state = AersoPhysicsState::default(); // You'll need to implement this

        physics.step(&mut state, 0.01).unwrap();

        // Verify forces were computed
        assert!(!physics.force_system.is_empty());

        // Verify state was updated
        assert!(state.spatial.velocity.norm() > 0.0);
    }
}
