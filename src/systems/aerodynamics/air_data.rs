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
    println!("Running air_data_system!");
    for (mut air_data, spatial) in query.iter_mut() {
        let calculation = AirDataCalculation::calculate(
            spatial,
            environment.get_wind(&spatial.position),
            environment.get_density(&spatial.position),
            config.min_airspeed_threshold,
        );
        println!("Spatial: {:?}, AirData: {:?}", &spatial, &air_data);

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

        // Compute angle of attack (alpha) and sideslip angle (beta)
        let (alpha, beta) = if airspeed > min_airspeed_threshold {
            (
                (relative_velocity.z / relative_velocity.x).atan(),
                (relative_velocity.y / airspeed).asin(),
            )
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

    #[allow(dead_code)]
    fn setup_test_app() {}

    #[allow(dead_code)]
    fn spawn_test_aircraft() {}

    #[test]
    fn test_stationary_aircraft() {}

    #[test]
    fn test_forward_flight() {}

    #[test]
    fn test_angle_of_attack() {}

    #[test]
    fn test_with_wind() {}

    #[test]
    fn test_altitude_density() {}
}
