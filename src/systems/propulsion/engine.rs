use crate::components::{PhysicsComponent, PropulsionComponent, SpatialComponent};
use crate::ecs::{System, World};
use crate::utils::errors::SimError;
use nalgebra::Vector3;

pub struct PropulsionSystemConfig {
    pub update_rate: f64,
    pub max_temp: f64,
    pub min_temp: f64,
    pub cooling_rate: f64,
}

impl Default for PropulsionSystemConfig {
    fn default() -> Self {
        Self {
            update_rate: 120.0,
            max_temp: 1000.0,
            min_temp: 288.15,
            cooling_rate: 0.1,
        }
    }
}

pub struct PropulsionSystem {
    config: PropulsionSystemConfig,
    accumulated_time: f64,
}

impl PropulsionSystem {
    pub fn new(config: PropulsionSystemConfig) -> Self {
        Self {
            config,
            accumulated_time: 0.0,
        }
    }

    fn compute_engine_forces(
        &self,
        throttle: f64,
        engine_params: &EngineParameters,
    ) -> Vector3<f64> {
        let thrust = throttle * engine_params.max_power * engine_params.efficiency;
        Vector3::new(thrust, 0.0, 0.0)
    }

    fn update_engine_temp(&self, current_temp: f64, throttle: f64, dt: f64) -> f64 {
        let target_temp =
            self.config.min_temp + (self.config.max_temp - self.config.min_temp) * throttle;

        let delta_temp = (target_temp - current_temp) * self.config.cooling_rate;
        current_temp + delta_temp * dt
    }
}

impl System for PropulsionSystem {
    fn update(&mut self, world: &mut World, dt: f64) -> Result<(), SimError> {
        self.accumulated_time += dt;

        if self.accumulated_time < (1.0 / self.config.update_rate) {
            return Ok(());
        }
        self.accumulated_time = 0.0;

        let query = world.query::<(
            &PropulsionComponent,
            &mut PhysicsComponent,
            &SpatialComponent,
        )>();

        for (entity, (propulsion, physics, spatial)) in query {
            let force = self.compute_engine_forces(propulsion.throttle, &propulsion.engine_params);

            let rotated_force = spatial.attitude * force;
            physics.add_force(rotated_force);

            if let Some(temp) = &mut propulsion.engine_temperature {
                *temp = self.update_engine_temp(*temp, propulsion.throttle, dt);
            }
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.accumulated_time = 0.0;
    }
}

struct EngineParameters {
    max_power: f64,
    efficiency: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_engine_forces() {
        let config = PropulsionSystemConfig::default();
        let system = PropulsionSystem::new(config);

        let engine_params = EngineParameters {
            max_power: 1000.0,
            efficiency: 0.8,
        };

        let force = system.compute_engine_forces(0.5, &engine_params);
        assert_eq!(force.x, 400.0);
        assert_eq!(force.y, 0.0);
        assert_eq!(force.z, 0.0);
    }

    #[test]
    fn test_engine_temp() {
        let config = PropulsionSystemConfig::default();
        let system = PropulsionSystem::new(config);

        let initial_temp = 300.0;
        let throttle = 0.8;
        let dt = 0.1;

        let new_temp = system.update_engine_temp(initial_temp, throttle, dt);
        assert!(new_temp > initial_temp);
        assert!(new_temp < config.max_temp);
    }
}
