use crate::components::{AirData, SpatialComponent};
use crate::resources::{AerodynamicsConfig, EnvironmentModel};
use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3}; // Make sure nalgebra is in scope
use std::f64::consts::PI; // Keep PI if clamping or other angle math is used

// --- Pure Calculation Logic ---

/// Simple struct to hold the results of air data calculation.
/// Can be returned by the pure function without returning the full Bevy component.
#[derive(Debug, Clone, Default)]
pub struct AirDataValues {
    pub true_airspeed: f64,
    pub alpha: f64, // Angle of Attack (rad)
    pub beta: f64,  // Sideslip Angle (rad)
    pub density: f64,
    pub dynamic_pressure: f64,
    pub relative_velocity_body: Vector3<f64>, // Velocity relative to air mass in body frame
}

/// Calculates air data based on spatial state and environment.
/// This is the "pure function" part.
///
/// # Arguments
/// * `velocity_inertial` - Aircraft velocity vector in the world/inertial frame.
/// * `attitude` - Aircraft orientation (Quaternion rotating from inertial to body frame).
/// * `wind_inertial` - Wind velocity vector in the world/inertial frame.
/// * `density` - Air density at the aircraft's position.
/// * `min_airspeed_threshold` - Minimum airspeed for valid alpha/beta calculation.
///
/// # Returns
/// An `AirDataValues` struct containing the calculated results.
pub fn calculate_air_data(
    velocity_inertial: &Vector3<f64>,
    attitude: &UnitQuaternion<f64>,
    wind_inertial: &Vector3<f64>,
    density: f64,
    min_airspeed_threshold: f64,
) -> AirDataValues {
    // Transform velocities to the body frame
    // attitude.inverse() transforms from inertial to body
    let velocity_body = attitude.inverse() * velocity_inertial;
    let wind_body = attitude.inverse() * wind_inertial;
    let relative_velocity_body = velocity_body - wind_body;

    // Compute true airspeed (magnitude of relative velocity)
    let airspeed = relative_velocity_body.norm();

    // Compute angle of attack (alpha) and sideslip angle (beta)
    let (alpha, beta) = if airspeed > min_airspeed_threshold {
        // atan2(y, x) gives angle from positive x-axis to point (x, y)
        // Body frame: +X forward, +Y right, +Z down
        // Alpha: Angle in vertical plane between body X-axis and relative velocity vector.
        //        Positive alpha means velocity vector is above body X-axis (nose down relative to airflow).
        //        Calculated as atan of (Vz_rel / Vx_rel). Use atan2 for quadrant safety.
        let mut alpha_rad = (relative_velocity_body.z).atan2(relative_velocity_body.x);

        // Beta: Angle in horizontal plane between body X-axis and relative velocity vector.
        //       Positive beta means velocity vector is to the right of body X-axis (airflow from left).
        //       Calculated as asin(Vy_rel / V_airspeed). Clamp input to asin to avoid NaN.
        let mut beta_rad = (relative_velocity_body.y / airspeed)
            .clamp(-1.0, 1.0)
            .asin();

        // Optional: Clamp results if needed for stability in specific contexts,
        // but often better to let downstream systems handle clamping if necessary.
        alpha_rad = alpha_rad.clamp(-30.0 * PI / 180.0, 30.0 * PI / 180.0);
        beta_rad = beta_rad.clamp(-30.0 * PI / 180.0, 30.0 * PI / 180.0);

        (alpha_rad, beta_rad)
    } else {
        (0.0, 0.0) // Default to zero if airspeed is too low
    };

    // Compute dynamic pressure (q = 0.5 * rho * V²)
    let dynamic_pressure = 0.5 * density * airspeed * airspeed;

    // Return the calculated values in the simple struct
    AirDataValues {
        true_airspeed: airspeed,
        alpha,
        beta,
        density,
        dynamic_pressure,
        relative_velocity_body,
    }
}

// --- Bevy System (Wrapper) ---

/// System responsible for calculating air data parameters for entities.
/// It queries components, calls the pure calculation function, and updates the AirData component.
pub fn air_data_system(
    mut query: Query<(&mut AirData, &SpatialComponent)>, // Query remains the same
    environment: Res<EnvironmentModel>,                  // Still need resources
    config: Res<AerodynamicsConfig>,
) {
    // println!("Running air_data_system!");
    for (mut air_data, spatial) in query.iter_mut() {
        // 1. Get Inputs needed for the pure function
        let wind_inertial = environment.get_wind(&spatial.position);
        let density = environment.get_density(&spatial.position);

        // 2. Call the pure calculation function
        let calculated_values: AirDataValues = calculate_air_data(
            &spatial.velocity,             // Pass inertial velocity from component
            &spatial.attitude,             // Pass attitude from component
            &wind_inertial,                // Pass world wind from environment
            density,                       // Pass density from environment
            config.min_airspeed_threshold, // Pass threshold from config
        );

        // 3. Update the AirData component with the results
        air_data.true_airspeed = calculated_values.true_airspeed;
        air_data.alpha = calculated_values.alpha;
        air_data.beta = calculated_values.beta;
        air_data.density = calculated_values.density;
        air_data.dynamic_pressure = calculated_values.dynamic_pressure;
        // Decide if you want to store relative_velocity_body in AirData component
        air_data.relative_velocity = calculated_values.relative_velocity_body;
        // Store world wind (as before)
        air_data.wind_velocity = wind_inertial;
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
        let result = AirDataCalculation::calculate(&spatial, wind, density, min_airspeed_threshold);

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
            velocity: Vector3::new(speed, 0.0, 0.0),   // Pure forward
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        // Standard sea level conditions
        let density = 1.225; // kg/m³
        let wind = Vector3::zeros();
        let min_airspeed_threshold = 0.5;

        // Run the calculation directly
        let result = AirDataCalculation::calculate(&spatial, wind, density, min_airspeed_threshold);

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
            let result =
                AirDataCalculation::calculate(&spatial, wind, density, min_airspeed_threshold);

            // Verify airspeed is correct
            assert_relative_eq!(
                result.true_airspeed,
                speed,
                epsilon = 1e-8,
                max_relative = 1e-8
            );

            // Verify alpha matches expected value (within small tolerance)
            assert_relative_eq!(result.alpha, radians, epsilon = 1e-8, max_relative = 1e-8);

            // Beta should still be 0
            assert_relative_eq!(result.beta, 0.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_with_wind() {
        // Test different wind conditions
        let wind_speed = 20.0;
        let headwind = Vector3::new(-wind_speed, 0.0, 0.0); // Wind flowing against aircraft
        let tailwind = Vector3::new(wind_speed, 0.0, 0.0); // Wind flowing with aircraft
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
            1.225,  // Sea level
            1.112,  // ~1000m
            0.7364, // ~5000m
            0.4135, // ~10000m
        ];

        let wind = Vector3::zeros();
        let min_airspeed_threshold = 0.5;

        for density in test_densities {
            // Run the calculation directly
            let result =
                AirDataCalculation::calculate(&spatial, wind, density, min_airspeed_threshold);

            // Verify density matches input
            assert_relative_eq!(result.density, density, epsilon = 1e-10);

            // Verify dynamic pressure is calculated correctly
            let expected_q = 0.5 * density * speed * speed;
            assert_relative_eq!(result.dynamic_pressure, expected_q, epsilon = 1e-10);
        }
    }
}
