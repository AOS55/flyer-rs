use crate::components::{AirData, SpatialComponent};
use crate::resources::{AerodynamicsConfig, EnvironmentModel};
use bevy::prelude::*;
use nalgebra::Vector3;

/// System responsible for calculating air data parameters for entities.
///
/// This system computes the airspeed, angle of attack (alpha), sideslip angle (beta),
/// and other parameters for each entity based on its spatial data and environmental conditions.
pub fn air_data_system(
    mut query: Query<(&mut AirData, &SpatialComponent)>,
    environment: Res<EnvironmentModel>,
    config: Res<AerodynamicsConfig>,
) {
    // println!("Running air_data_system!");
    for (mut air_data, spatial) in query.iter_mut() {
        let calculation = AirDataCalculation::calculate(
            spatial,
            environment.get_wind(&spatial.position),
            environment.get_density(&spatial.position),
            config.min_airspeed_threshold,
        );
        // println!("Spatial: {:?}, AirData: {:?}", &spatial, &air_data);

        // Update air data
        air_data.true_airspeed = calculation.true_airspeed;
        air_data.alpha = calculation.alpha;
        air_data.beta = calculation.beta;
        air_data.density = calculation.density;
        air_data.dynamic_pressure = calculation.dynamic_pressure;
        air_data.relative_velocity = calculation.relative_velocity;
        air_data.wind_velocity = calculation.wind_velocity;
    }
}

/// Helper structure for performing air data calculations.
///
/// Encapsulates the results of calculations for airspeed, alpha, beta, and other
/// air data metrics. Provides a reusable calculation function for modularity and clarity.
#[derive(Debug)]
struct AirDataCalculation {
    /// True airspeed of the entity in m/s.
    true_airspeed: f64,
    /// Angle of attack (alpha) in radians.
    alpha: f64,
    /// Sideslip angle (beta) in radians.
    beta: f64,
    /// Air density at the entity's position in kg/m³.
    density: f64,
    /// Dynamic pressure in Pa.
    dynamic_pressure: f64,
    /// Relative velocity of the entity in the body frame in m/s.
    relative_velocity: Vector3<f64>,
    /// Wind velocity at the entity's position in the body frame in m/s.
    wind_velocity: Vector3<f64>,
}

impl AirDataCalculation {
    /// Performs air data calculations based on spatial properties, wind, and density.
    ///
    /// # Arguments
    /// * `spatial` - The spatial component containing position, velocity, and attitude.
    /// * `wind` - Wind velocity vector at the entity's position in the world frame.
    /// * `density` - Air density at the entity's position.
    /// * `min_airspeed_threshold` - Minimum airspeed threshold for valid alpha and beta calculations.
    ///
    /// # Returns
    /// An instance of `AirDataCalculation` containing the computed values.
    fn calculate(
        spatial: &SpatialComponent,
        wind: Vector3<f64>,
        density: f64,
        min_airspeed_threshold: f64,
    ) -> Self {
        // Transform velocities to the body frame
        let velocity_body = spatial.attitude.inverse() * spatial.velocity;
        let wind_body = spatial.attitude.inverse() * wind;
        let relative_velocity = velocity_body - wind_body;

        // Compute true airspeed (magnitude of relative velocity)
        let airspeed = relative_velocity.norm();

        // Compute angle of attack (alpha) and sideslip angle (beta) more accurately
        let (alpha, beta) = if airspeed > min_airspeed_threshold {
            // Calculate angle of attack properly using the inverse tangent of vertical/forward velocities
            // This handles large angles correctly without small-angle approximation
            let alpha = (-relative_velocity.z).atan2(relative_velocity.x);

            // Calculate sideslip using the inverse sine, but constrain to valid range
            let beta = (relative_velocity.y / airspeed).clamp(-1.0, 1.0).asin();

            (alpha, beta)
        } else {
            (0.0, 0.0)
        };

        // Compute dynamic pressure (q = 0.5 * rho * V²)
        let dynamic_pressure = 0.5 * density * airspeed * airspeed;

        // Return computed values encapsulated in `AirDataCalculation`
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
    use crate::resources::{AerodynamicsConfig, EnvironmentConfig, EnvironmentModel};
    use approx::assert_relative_eq;
    use nalgebra::UnitQuaternion;
    use std::f64::consts::PI;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.insert_resource(AerodynamicsConfig {
            min_airspeed_threshold: 0.5,
        });
        app.insert_resource(EnvironmentModel::new(&EnvironmentConfig::default()));
        app
    }

    fn spawn_test_aircraft(app: &mut App, spatial: SpatialComponent) -> Entity {
        app.world_mut().spawn((spatial, AirData::default())).id()
    }

    #[test]
    fn test_stationary_aircraft() {
        // Create aircraft with zero velocity
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0), // 1000m altitude
            velocity: Vector3::zeros(),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };
        
        // Standard sea level conditions
        let density = 1.225; // kg/m³
        let wind = Vector3::zeros();
        let min_airspeed_threshold = 0.5;
        
        // Run the calculation directly
        let result = AirDataCalculation::calculate(
            &spatial,
            wind,
            density,
            min_airspeed_threshold
        );
        
        // For a stationary aircraft with no wind:
        // - Airspeed should be 0
        // - Alpha and beta should be 0 (or undefined, but we default to 0)
        assert_relative_eq!(result.true_airspeed, 0.0);
        assert_relative_eq!(result.alpha, 0.0);
        assert_relative_eq!(result.beta, 0.0);
        assert_relative_eq!(result.density, density);
    }

    #[test]
    fn test_forward_flight() {
        // Create aircraft with pure forward velocity
        let speed = 100.0;
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0), // 1000m altitude
            velocity: Vector3::new(speed, 0.0, 0.0),  // Pure forward
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };
        
        // Standard sea level conditions
        let density = 1.225; // kg/m³
        let wind = Vector3::zeros();
        let min_airspeed_threshold = 0.5;
        
        // Run the calculation directly
        let result = AirDataCalculation::calculate(
            &spatial,
            wind,
            density,
            min_airspeed_threshold
        );
        
        // For pure forward flight:
        // - Airspeed should match velocity magnitude
        // - Alpha and beta should be 0
        assert_relative_eq!(result.true_airspeed, speed, epsilon = 1e-10);
        assert_relative_eq!(result.alpha, 0.0, epsilon = 1e-10);
        assert_relative_eq!(result.beta, 0.0, epsilon = 1e-10);
        
        // Dynamic pressure should be calculated correctly
        let expected_q = 0.5 * density * speed * speed;
        assert_relative_eq!(result.dynamic_pressure, expected_q, epsilon = 1e-10);
    }

    #[test]
    fn test_angle_of_attack() {
        // Test several angles of attack
        let test_angles = vec![0.0, 5.0, 10.0, -5.0, 15.0, 30.0, -10.0];
        
        // Standard sea level conditions
        let density = 1.225; // kg/m³
        let wind = Vector3::zeros();
        let min_airspeed_threshold = 0.5;
        
        for degrees in test_angles {
            let radians = degrees * PI / 180.0;
            let speed = 100.0;
            
            // Set velocity components to achieve desired alpha
            // For alpha = arctan(-vz/vx), set vx and vz accordingly
            let vx = speed * radians.cos();
            let vz = -speed * radians.sin(); // Negative because Z is down
            
            let spatial = SpatialComponent {
                position: Vector3::new(0.0, 0.0, -1000.0),
                velocity: Vector3::new(vx, 0.0, vz),
                attitude: UnitQuaternion::identity(),
                angular_velocity: Vector3::zeros(),
            };
            
            // Run the calculation directly
            let result = AirDataCalculation::calculate(
                &spatial,
                wind,
                density,
                min_airspeed_threshold
            );
            
            // Verify airspeed is correct
            assert_relative_eq!(
                result.true_airspeed, 
                speed, 
                epsilon = 1e-8, 
                max_relative = 1e-8
            );
            
            // Verify alpha matches expected value (within small tolerance)
            assert_relative_eq!(
                result.alpha, 
                radians, 
                epsilon = 1e-8, 
                max_relative = 1e-8
            );
            
            // Beta should still be 0
            assert_relative_eq!(result.beta, 0.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_with_wind() {
        // Test different wind conditions
        let wind_speed = 20.0;
        let headwind = Vector3::new(-wind_speed, 0.0, 0.0); // Wind flowing against aircraft
        let tailwind = Vector3::new(wind_speed, 0.0, 0.0);  // Wind flowing with aircraft
        let crosswind = Vector3::new(0.0, wind_speed, 0.0); // Wind from the side
        
        // Standard sea level conditions
        let _density = 1.225; // kg/m³
        let _min_airspeed_threshold = 0.5;
        
        // Test cases for different wind conditions
        let test_cases = vec![
            ("headwind", headwind),
            ("tailwind", tailwind),
            ("crosswind", crosswind),
        ];
        
        for (wind_name, wind) in test_cases {
            // Create aircraft with forward velocity
            let aircraft_speed = 100.0;
            let spatial = SpatialComponent {
                position: Vector3::new(0.0, 0.0, -1000.0),
                velocity: Vector3::new(aircraft_speed, 0.0, 0.0), // Pure forward
                attitude: UnitQuaternion::identity(),
                angular_velocity: Vector3::zeros(),
            };
            
            // Manually calculate expected results
            let relative_velocity = spatial.velocity - wind;
            let expected_airspeed = relative_velocity.norm();

            // For alpha and beta calculations
            let expected_alpha = if expected_airspeed > 0.5 {
                (-relative_velocity.z).atan2(relative_velocity.x)
            } else {
                0.0
            };

            let expected_beta = if expected_airspeed > 0.5 {
                (relative_velocity.y / expected_airspeed)
                    .clamp(-1.0, 1.0)
                    .asin()
            } else {
                0.0
            };

            // Run the calculation code from AirDataCalculation directly
            let calculation = AirDataCalculation::calculate(
                &spatial, wind, 1.225, // Sea level density
                0.5,   // Min airspeed threshold
            );

            // Verify results
            assert_relative_eq!(
                calculation.true_airspeed,
                expected_airspeed,
                epsilon = 1e-6,
                max_relative = 1e-6
            );

            assert_relative_eq!(
                calculation.alpha,
                expected_alpha,
                epsilon = 1e-6,
                max_relative = 1e-6
            );

            assert_relative_eq!(
                calculation.beta,
                expected_beta,
                epsilon = 1e-6,
                max_relative = 1e-6
            );

            println!(
                "Wind case '{}' passed: airspeed={}, alpha={}, beta={}",
                wind_name, calculation.true_airspeed, calculation.alpha, calculation.beta
            );
        }
    }

    #[test]
    fn test_altitude_effects() {
        // Create basic spatial component
        let speed = 100.0;
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0), // 1000m altitude
            velocity: Vector3::new(speed, 0.0, 0.0),
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };
        
        // Test different densities (representing different altitudes)
        let test_densities = vec![
            1.225,   // Sea level
            1.112,   // ~1000m
            0.7364,  // ~5000m
            0.4135,  // ~10000m
        ];
        
        let wind = Vector3::zeros();
        let min_airspeed_threshold = 0.5;
        
        for density in test_densities {
            // Run the calculation directly
            let result = AirDataCalculation::calculate(
                &spatial,
                wind,
                density,
                min_airspeed_threshold
            );
            
            // Verify density matches input
            assert_relative_eq!(result.density, density, epsilon = 1e-10);
            
            // Verify dynamic pressure is calculated correctly
            let expected_q = 0.5 * density * speed * speed;
            assert_relative_eq!(
                result.dynamic_pressure, 
                expected_q, 
                epsilon = 1e-10
            );
        }
    }
}
