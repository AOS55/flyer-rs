use crate::components::{AerodynamicsComponent, SpatialComponent};
use crate::config::aerodynamics::AerodynamicsConfig;
use crate::resources::EnvironmentResource;
use bevy::prelude::*;
use nalgebra::Vector3;

/// System for calculating air data parameters
pub fn air_data_system(
    mut query: Query<(&mut AerodynamicsComponent, &SpatialComponent)>,
    environment: Res<EnvironmentResource>,
    config: Res<AerodynamicsConfig>,
) {
    for (mut aero, spatial) in query.iter_mut() {
        let calculation = AirDataCalculation::calculate(
            spatial,
            environment.get_wind(&spatial.position),
            environment.get_density(&spatial.position),
            config.min_airspeed_threshold,
        );

        // Update air data
        aero.air_data.true_airspeed = calculation.true_airspeed;
        aero.air_data.alpha = calculation.alpha;
        aero.air_data.beta = calculation.beta;
        aero.air_data.density = calculation.density;
        aero.air_data.dynamic_pressure = calculation.dynamic_pressure;
        aero.air_data.relative_velocity = calculation.relative_velocity;
        aero.air_data.wind_velocity = calculation.wind_velocity;
    }
}

/// Helper struct for air data calculations
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
    fn calculate(
        spatial: &SpatialComponent,
        wind: Vector3<f64>,
        density: f64,
        min_airspeed_threshold: f64,
    ) -> Self {
        // Calculate relative velocity in body frame
        let velocity_body = spatial.attitude.inverse() * spatial.velocity;
        let wind_body = spatial.attitude.inverse() * wind;
        let relative_velocity = velocity_body - wind_body;
        let airspeed = relative_velocity.norm();

        // Calculate alpha and beta
        let (alpha, beta) = if airspeed > min_airspeed_threshold {
            (
                (relative_velocity.z / relative_velocity.x).atan(),
                (relative_velocity.y / airspeed).asin(),
            )
        } else {
            (0.0, 0.0)
        };

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::AircraftGeometry;
    use crate::resources::{AtmosphereConfig, AtmosphereType, EnvironmentConfig, WindModelConfig};
    use approx::assert_relative_eq;
    use nalgebra::{UnitQuaternion, Vector3};
    use std::f64::consts::PI;

    fn setup_test_app() -> App {
        let mut app = App::new();

        // Add required resources
        app.init_resource::<AerodynamicsConfig>();

        // Add environment resource with zero wind
        let env_config = EnvironmentConfig {
            wind_model_config: WindModelConfig::Constant {
                velocity: Vector3::zeros(),
            },
            atmosphere_config: AtmosphereConfig {
                model_type: AtmosphereType::Standard,
                sea_level_density: 1.225,
                sea_level_temperature: 288.15,
            },
        };
        app.insert_resource(EnvironmentResource::new(&env_config));

        app
    }

    fn spawn_test_aircraft(
        app: &mut App,
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        attitude: UnitQuaternion<f64>,
    ) -> Entity {
        app.world
            .spawn((
                SpatialComponent {
                    position,
                    velocity,
                    attitude,
                    angular_velocity: Vector3::zeros(),
                },
                AerodynamicsComponent {
                    geometry: AircraftGeometry::default(),
                    ..Default::default()
                },
            ))
            .id()
    }

    #[test]
    fn test_stationary_aircraft() {
        let mut app = setup_test_app();

        let entity = spawn_test_aircraft(
            &mut app,
            Vector3::zeros(),
            Vector3::zeros(),
            UnitQuaternion::identity(),
        );

        // Run the system
        app.add_systems(Update, air_data_system);
        app.update();

        // Check results
        let aero = app.world.get::<AerodynamicsComponent>(entity).unwrap();

        assert!(aero.air_data.true_airspeed < 1e-6);
        assert_relative_eq!(aero.air_data.alpha, 0.0);
        assert_relative_eq!(aero.air_data.beta, 0.0);
        assert_relative_eq!(aero.air_data.density, 1.225, epsilon = 0.001);
        assert_relative_eq!(aero.air_data.dynamic_pressure, 0.0);
    }

    #[test]
    fn test_forward_flight() {
        let mut app = setup_test_app();

        let entity = spawn_test_aircraft(
            &mut app,
            Vector3::zeros(),
            Vector3::new(50.0, 0.0, 0.0), // 50 m/s forward flight
            UnitQuaternion::identity(),
        );

        // Run the system
        app.add_systems(Update, air_data_system);
        app.update();

        // Check results
        let aero = app.world.get::<AerodynamicsComponent>(entity).unwrap();

        assert_relative_eq!(aero.air_data.true_airspeed, 50.0, epsilon = 0.001);
        assert_relative_eq!(aero.air_data.alpha, 0.0, epsilon = 0.001);
        assert_relative_eq!(aero.air_data.beta, 0.0, epsilon = 0.001);
        assert_relative_eq!(
            aero.air_data.dynamic_pressure,
            0.5 * 1.225 * 50.0 * 50.0,
            epsilon = 0.001
        );
    }

    #[test]
    fn test_angle_of_attack() {
        let mut app = setup_test_app();

        // Create aircraft with velocity components giving 10 degrees AoA
        let velocity = Vector3::new(
            50.0 * f64::cos(10.0 * PI / 180.0),
            0.0,
            50.0 * f64::sin(10.0 * PI / 180.0),
        );

        let entity = spawn_test_aircraft(
            &mut app,
            Vector3::zeros(),
            velocity,
            UnitQuaternion::identity(),
        );

        // Run the system
        app.add_systems(Update, air_data_system);
        app.update();

        // Check results
        let aero = app.world.get::<AerodynamicsComponent>(entity).unwrap();

        assert_relative_eq!(aero.air_data.alpha, 10.0 * PI / 180.0, epsilon = 0.001);
        assert_relative_eq!(aero.air_data.beta, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_with_wind() {
        let mut app = App::new();

        // Add config
        app.init_resource::<AerodynamicsConfig>();

        // Add environment with constant wind
        let env_config = EnvironmentConfig {
            wind_model_config: WindModelConfig::Constant {
                velocity: Vector3::new(10.0, 0.0, 0.0), // 10 m/s headwind
            },
            atmosphere_config: AtmosphereConfig::default(),
        };
        app.insert_resource(EnvironmentResource::new(&env_config));

        // Spawn aircraft with forward velocity
        let entity = spawn_test_aircraft(
            &mut app,
            Vector3::zeros(),
            Vector3::new(50.0, 0.0, 0.0), // 50 m/s forward
            UnitQuaternion::identity(),
        );

        // Run the system
        app.add_systems(Update, air_data_system);
        app.update();

        // Check results
        let aero = app.world.get::<AerodynamicsComponent>(entity).unwrap();

        // True airspeed should be ground speed + headwind
        assert_relative_eq!(aero.air_data.true_airspeed, 60.0, epsilon = 0.001);
        assert_relative_eq!(aero.air_data.wind_velocity.x, 10.0, epsilon = 0.001);
    }

    #[test]
    fn test_altitude_density() {
        let mut app = setup_test_app();

        // Create aircraft at different altitudes
        let entities = vec![
            spawn_test_aircraft(
                &mut app,
                Vector3::new(0.0, 0.0, 0.0), // Sea level
                Vector3::new(50.0, 0.0, 0.0),
                UnitQuaternion::identity(),
            ),
            spawn_test_aircraft(
                &mut app,
                Vector3::new(0.0, 0.0, 5000.0), // 5000m altitude
                Vector3::new(50.0, 0.0, 0.0),
                UnitQuaternion::identity(),
            ),
        ];

        // Run the system
        app.add_systems(Update, air_data_system);
        app.update();

        // Check results
        let sea_level_density = app
            .world
            .get::<AerodynamicsComponent>(entities[0])
            .unwrap()
            .air_data
            .density;
        let altitude_density = app
            .world
            .get::<AerodynamicsComponent>(entities[1])
            .unwrap()
            .air_data
            .density;

        assert!(
            altitude_density < sea_level_density,
            "Density should decrease with altitude"
        );
        assert_relative_eq!(sea_level_density, 1.225, epsilon = 0.001);
    }
}
