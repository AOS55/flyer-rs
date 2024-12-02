use crate::components::{AerodynamicsComponent, SpatialComponent};
use crate::ecs::error::Result;
use crate::ecs::{EcsError, EntityId, System, World};
use crate::resources::EnvironmentResource;
use nalgebra::Vector3;

pub struct AirDataSystem;

const MIN_AIRSPEED_THRESHOLD: f64 = 1e-6;

#[derive(Debug)]
struct AirDataCalculation {
    true_airspeed: f64,
    alpha: f64,
    beta: f64,
    density: f64,
    dynamic_pressure: f64,
    relative_velocity: Vector3<f64>,
    wind_velocity: Vector3<f64>,
}

impl AirDataCalculation {
    fn calculate(spatial: &SpatialComponent, wind: Vector3<f64>, density: f64) -> Self {
        // Calculate relative velocity in body frame
        let velocity_body = spatial.attitude.inverse() * spatial.velocity;
        let wind_body = spatial.attitude.inverse() * wind;
        let relative_velocity = velocity_body - wind_body;
        let airspeed = relative_velocity.norm();

        // Use more descriptive calculation methods
        let alpha = Self::calculate_alpha(&relative_velocity, airspeed);
        let beta = Self::calculate_beta(&relative_velocity, airspeed);
        let dynamic_pressure = 0.5 * density * airspeed * airspeed;

        Self {
            true_airspeed: airspeed,
            alpha,
            beta,
            density,
            dynamic_pressure,
            relative_velocity,
            wind_velocity: wind,
        }
    }

    fn calculate_alpha(relative_velocity: &Vector3<f64>, airspeed: f64) -> f64 {
        if airspeed > MIN_AIRSPEED_THRESHOLD {
            (relative_velocity.z / relative_velocity.x).atan()
        } else {
            0.0
        }
    }

    fn calculate_beta(relative_velocity: &Vector3<f64>, airspeed: f64) -> f64 {
        if airspeed > MIN_AIRSPEED_THRESHOLD {
            (relative_velocity.y / airspeed).asin()
        } else {
            0.0
        }
    }
}

impl System for AirDataSystem {
    fn name(&self) -> &str {
        "Air Data System"
    }

    fn run(&mut self, world: &mut World) -> Result<()> {
        // Structure to hold the calculation data
        struct UpdateData {
            entity_id: EntityId,
            calculation: AirDataCalculation,
        }

        // First pass: collect all necessary data
        let mut updates = Vec::new();

        {
            let env = world.get_resource::<EnvironmentResource>()?;

            for (entity, spatial) in world.query::<SpatialComponent>() {
                // Only process entities that have both components
                if world.has_component::<AerodynamicsComponent>(entity) {
                    let wind = env.get_wind(&spatial.position);
                    let density = env.get_density(&spatial.position);

                    let calculation = AirDataCalculation::calculate(spatial, wind, density);

                    updates.push(UpdateData {
                        entity_id: entity,
                        calculation,
                    });
                }
            }
        }

        // Second pass: apply updates
        for update in updates {
            let aero = world
                .get_component_mut::<AerodynamicsComponent>(update.entity_id)
                .map_err(|_| {
                    EcsError::ComponentError(format!(
                        "Failed to get AerodynamicsComponent for entity {:?}",
                        update.entity_id
                    ))
                })?;

            aero.air_data.true_airspeed = update.calculation.true_airspeed;
            aero.air_data.alpha = update.calculation.alpha;
            aero.air_data.beta = update.calculation.beta;
            aero.air_data.density = update.calculation.density;
            aero.air_data.dynamic_pressure = update.calculation.dynamic_pressure;
            aero.air_data.relative_velocity = update.calculation.relative_velocity;
            aero.air_data.wind_velocity = update.calculation.wind_velocity;
        }

        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        // Specify system dependencies if any
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{AerodynamicsComponent, SpatialComponent};
    use crate::ecs::World;
    use crate::resources::{
        AtmosphereConfig, AtmosphereType, EnvironmentConfig, EnvironmentResource, WindModelConfig,
    };
    use approx::assert_relative_eq;
    use nalgebra::{UnitQuaternion, Vector3};
    use std::f64::consts::PI;

    fn setup_test_world() -> World {
        let mut world = World::new();

        let env_config = EnvironmentConfig {
            wind_model_config: WindModelConfig::Constant {
                velocity: Vector3::zeros(), // Start with zero wind for simplicity
            },
            atmosphere_config: AtmosphereConfig {
                model_type: AtmosphereType::Standard,
                sea_level_density: 1.225,
                sea_level_temperature: 288.15,
            },
        };

        let env = EnvironmentResource::new(&env_config);
        world.add_resource(env).unwrap();

        world
    }

    fn create_test_entity(
        world: &mut World,
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        attitude: UnitQuaternion<f64>,
    ) -> EntityId {
        let entity = world.spawn();

        let spatial = SpatialComponent {
            position,
            velocity,
            attitude,
            angular_velocity: Vector3::zeros(),
        };

        let aero = AerodynamicsComponent::default();

        world.add_component(entity, spatial).unwrap();
        world.add_component(entity, aero).unwrap();

        entity
    }

    #[test]
    fn test_stationary_aircraft() {
        let mut world = setup_test_world();

        // Create a stationary aircraft at some altitude
        let position = Vector3::new(0.0, 0.0, 0.0);
        let velocity = Vector3::zeros();
        let attitude = UnitQuaternion::identity();

        let entity = create_test_entity(&mut world, position, velocity, attitude);

        let mut system = AirDataSystem;
        system.run(&mut world).unwrap();

        let aero = world
            .get_component::<AerodynamicsComponent>(entity)
            .unwrap();

        // For a stationary aircraft:
        assert!(aero.air_data.true_airspeed < MIN_AIRSPEED_THRESHOLD);
        assert_relative_eq!(aero.air_data.alpha, 0.0);
        assert_relative_eq!(aero.air_data.beta, 0.0);
        assert_relative_eq!(aero.air_data.density, 1.225, epsilon = 0.001);
        assert_relative_eq!(aero.air_data.dynamic_pressure, 0.0);
    }

    #[test]
    fn test_logarithmic_wind() {
        let mut world = World::new();

        // Create environment with logarithmic wind profile
        let env_config = EnvironmentConfig {
            wind_model_config: WindModelConfig::Logarithmic {
                d: 0.0,       // Zero plane displacement
                z0: 0.03,     // Typical surface roughness
                u_star: 0.5,  // Friction velocity
                bearing: 0.0, // Wind from North
            },
            atmosphere_config: AtmosphereConfig::default(),
        };
        let env = EnvironmentResource::new(&env_config);
        world.add_resource(env).unwrap();

        // Create aircraft at two different altitudes
        // Note: For the wind calculation, we'll let the EnvironmentResource handle the z-coordinate conversion
        let entity1 = create_test_entity(
            &mut world,
            Vector3::new(0.0, 0.0, -10.0), // 10m altitude
            Vector3::zeros(),
            UnitQuaternion::identity(),
        );

        let entity2 = create_test_entity(
            &mut world,
            Vector3::new(0.0, 0.0, -100.0), // 100m altitude
            Vector3::zeros(),
            UnitQuaternion::identity(),
        );

        // First, let's verify the wind calculations directly using the environment resource
        let env = world.get_resource::<EnvironmentResource>().unwrap();
        let wind1 = env.get_wind(&Vector3::new(0.0, 0.0, -10.0));
        let wind2 = env.get_wind(&Vector3::new(0.0, 0.0, -100.0));

        println!("Direct wind calculation:");
        println!("Wind at 10m altitude: {:?}", wind1);
        println!("Wind at 100m altitude: {:?}", wind2);

        let mut system = AirDataSystem;
        system.run(&mut world).unwrap();

        let aero1 = world
            .get_component::<AerodynamicsComponent>(entity1)
            .unwrap();
        let aero2 = world
            .get_component::<AerodynamicsComponent>(entity2)
            .unwrap();

        println!("\nThrough AirDataSystem:");
        println!("Wind at 10m altitude: {:?}", aero1.air_data.wind_velocity);
        println!("Wind at 100m altitude: {:?}", aero2.air_data.wind_velocity);
        println!("Airspeed at 10m: {}", aero1.air_data.true_airspeed);
        println!("Airspeed at 100m: {}", aero2.air_data.true_airspeed);

        // Verify that we're getting valid wind values
        assert!(!aero1.air_data.wind_velocity.x.is_nan());
        assert!(!aero2.air_data.wind_velocity.x.is_nan());

        // Wind speed should increase with altitude in a logarithmic profile
        assert!(
            aero2.air_data.true_airspeed > aero1.air_data.true_airspeed,
            "Expected wind speed at 100m ({}) to be greater than at 10m ({})",
            aero2.air_data.true_airspeed,
            aero1.air_data.true_airspeed
        );

        // The wind should be from the North (positive x in NED coordinates)
        assert!(
            aero1.air_data.wind_velocity.x > 0.0,
            "Expected positive wind from North, got: {}",
            aero1.air_data.wind_velocity.x
        );

        // No crosswind component
        assert_relative_eq!(aero1.air_data.wind_velocity.y, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_density_variation() {
        let mut world = setup_test_world();

        // Test aircraft at different altitudes
        let altitudes = vec![0.0, 1000.0, 5000.0, 10000.0];
        let mut entities = Vec::new();

        for altitude in altitudes {
            let entity = create_test_entity(
                &mut world,
                Vector3::new(0.0, 0.0, altitude),
                Vector3::new(50.0, 0.0, 0.0), // Forward flight
                UnitQuaternion::identity(),
            );
            entities.push(entity);
        }

        let mut system = AirDataSystem;
        system.run(&mut world).unwrap();

        // Get densities at each altitude
        let densities: Vec<f64> = entities
            .iter()
            .map(|&e| {
                world
                    .get_component::<AerodynamicsComponent>(e)
                    .unwrap()
                    .air_data
                    .density
            })
            .collect();

        // Verify density decreases with altitude
        for i in 0..densities.len() - 1 {
            assert!(
                densities[i] > densities[i + 1],
                "Density should decrease with altitude"
            );
        }

        // Verify sea level density is close to standard value
        assert_relative_eq!(densities[0], 1.225, epsilon = 0.001);

        // Verify dynamic pressure changes with density
        for (i, &entity) in entities.iter().enumerate() {
            let aero = world
                .get_component::<AerodynamicsComponent>(entity)
                .unwrap();
            let expected_q = 0.5 * densities[i] * 50.0 * 50.0;
            assert_relative_eq!(aero.air_data.dynamic_pressure, expected_q, epsilon = 0.001);
        }
    }

    #[test]
    fn test_angle_of_attack_calculation() {
        let mut world = setup_test_world();

        // Test different flight conditions
        let test_cases = vec![
            // (velocity_x, velocity_z, expected_alpha)
            (50.0, 0.0, 0.0),                  // Level flight
            (50.0, 8.75, 10.0 * PI / 180.0),   // 10 degrees climb
            (50.0, -8.75, -10.0 * PI / 180.0), // 10 degrees descent
        ];

        for (vx, vz, expected_alpha) in test_cases {
            let entity = create_test_entity(
                &mut world,
                Vector3::new(0.0, 0.0, 1000.0),
                Vector3::new(vx, 0.0, vz),
                UnitQuaternion::identity(),
            );

            let mut system = AirDataSystem;
            system.run(&mut world).unwrap();

            let aero = world
                .get_component::<AerodynamicsComponent>(entity)
                .unwrap();

            // Verify angle of attack calculation
            assert!((aero.air_data.alpha - expected_alpha).abs() < 1e-2);
        }
    }
}
