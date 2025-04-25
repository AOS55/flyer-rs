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
    let min_threshold = config.min_airspeed_threshold;
    // println!("Running air_data_system!");
    query.par_iter_mut().for_each(|(mut air_data, spatial)| {
        // 1. Get Inputs needed for the pure function
        let wind_inertial = environment.get_wind(&spatial.position);
        let density = environment.get_density(&spatial.position);

        // 2. Call the pure calculation function
        let calculated_values: AirDataValues = calculate_air_data(
            &spatial.velocity, // Pass inertial velocity from component
            &spatial.attitude, // Pass attitude from component
            &wind_inertial,    // Pass world wind from environment
            density,           // Pass density from environment
            min_threshold,     // Pass threshold from config
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
    });
}

#[cfg(test)]
mod tests {
    use super::*; // Imports calculate_air_data, AirDataValues
    use approx::assert_relative_eq;
    use nalgebra::{UnitQuaternion, Vector3};
    use std::f64::consts::PI;

    // --- Constants for Tests ---
    const STD_DENSITY: f64 = 1.225; // Standard sea level density
    const TEST_EPSILON: f64 = 1e-9; // Tolerance for floating point comparisons
    const MIN_AIRSPEED_THRESHOLD: f64 = 0.5; // Default threshold for tests

    #[test]
    fn test_stationary_no_wind() {
        // Scenario: Aircraft stationary, no wind
        let velocity_inertial = Vector3::zeros();
        let attitude = UnitQuaternion::identity();
        let wind_inertial = Vector3::zeros();
        let density = STD_DENSITY;

        let result = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );

        assert_relative_eq!(result.true_airspeed, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.alpha, 0.0, epsilon = TEST_EPSILON); // Below threshold defaults to 0
        assert_relative_eq!(result.beta, 0.0, epsilon = TEST_EPSILON); // Below threshold defaults to 0
        assert_relative_eq!(result.density, density, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.dynamic_pressure, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result.relative_velocity_body,
            Vector3::zeros(),
            epsilon = TEST_EPSILON
        );
    }

    #[test]
    fn test_forward_flight_no_wind() {
        // Scenario: Pure forward flight, no wind, level attitude
        let speed = 100.0;
        let velocity_inertial = Vector3::new(speed, 0.0, 0.0); // Forward in inertial frame
        let attitude = UnitQuaternion::identity(); // Level flight
        let wind_inertial = Vector3::zeros();
        let density = STD_DENSITY;

        let result = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );

        assert_relative_eq!(result.true_airspeed, speed, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.alpha, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.beta, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.density, density, epsilon = TEST_EPSILON);
        let expected_q = 0.5 * density * speed * speed;
        assert_relative_eq!(result.dynamic_pressure, expected_q, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result.relative_velocity_body,
            Vector3::new(speed, 0.0, 0.0), // Should be same as inertial velocity in body frame here
            epsilon = TEST_EPSILON
        );
    }

    #[test]
    fn test_positive_alpha_no_wind() {
        // Scenario: Velocity vector angled upwards relative to body X-axis (positive alpha)
        // Body frame: +X fwd, +Y right, +Z down
        // Alpha: atan2(Vz_rel, Vx_rel). Positive alpha means Vz_rel > 0 (velocity points "down" in body space relative to body X)
        // OR: Aircraft nose is below the relative wind vector.
        let speed = 100.0;
        let angle_deg = 10.0;
        let angle_rad = angle_deg * PI / 180.0;

        // To get +10 deg alpha (atan2(vz, vx)), need vz > 0, vx > 0.
        // Let velocity in body frame be [cos(alpha), 0, sin(alpha)] * speed
        let vx_body = speed * angle_rad.cos();
        let vz_body = speed * angle_rad.sin();
        let velocity_body = Vector3::new(vx_body, 0.0, vz_body);

        // Assume level attitude for simplicity, so body velocity = inertial velocity
        let velocity_inertial = velocity_body;
        let attitude = UnitQuaternion::identity();
        let wind_inertial = Vector3::zeros();
        let density = STD_DENSITY;

        let result = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );

        assert_relative_eq!(result.true_airspeed, speed, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.alpha, angle_rad, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.beta, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result.relative_velocity_body,
            velocity_body,
            epsilon = TEST_EPSILON
        );
    }

    #[test]
    fn test_positive_beta_no_wind() {
        // Scenario: Velocity vector angled right relative to body X-axis (positive beta)
        // Body frame: +X fwd, +Y right, +Z down
        // Beta: asin(Vy_rel / airspeed). Positive beta means Vy_rel > 0 (velocity points "right" in body space relative to body X).
        // OR: Relative wind is coming from the left of the aircraft nose.
        let speed = 100.0;
        let angle_deg = 10.0;
        let angle_rad = angle_deg * PI / 180.0;

        // To get +10 deg beta (asin(vy / speed)), need vy > 0.
        // Let velocity in body frame be [cos(beta), sin(beta), 0] * speed (approximation for small angles, safer to use components)
        let vx_body = speed * angle_rad.cos(); // Component along X
        let vy_body = speed * angle_rad.sin(); // Component along Y

        let velocity_body = Vector3::new(vx_body, vy_body, 0.0);
        // Ensure magnitude is correct (it is, due to trig identities)
        assert_relative_eq!(velocity_body.norm(), speed, epsilon = TEST_EPSILON);

        // Assume level attitude for simplicity, so body velocity = inertial velocity
        let velocity_inertial = velocity_body;
        let attitude = UnitQuaternion::identity();
        let wind_inertial = Vector3::zeros();
        let density = STD_DENSITY;

        let result = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );

        assert_relative_eq!(result.true_airspeed, speed, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.alpha, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.beta, angle_rad, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result.relative_velocity_body,
            velocity_body,
            epsilon = TEST_EPSILON
        );
    }

    #[test]
    fn test_wind_effects() {
        let aircraft_speed_inertial = 100.0;
        let wind_speed = 20.0;
        let velocity_inertial = Vector3::new(aircraft_speed_inertial, 0.0, 0.0);
        let attitude = UnitQuaternion::identity();
        let density = STD_DENSITY;

        // --- Headwind ---
        let headwind = Vector3::new(-wind_speed, 0.0, 0.0); // Wind towards -X
        let result_head = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &headwind,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        // Vrel_body = Vair_body - Vwind_body = [100,0,0] - [-20,0,0] = [120,0,0]
        assert_relative_eq!(result_head.true_airspeed, 120.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result_head.alpha, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result_head.beta, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result_head.relative_velocity_body,
            Vector3::new(120.0, 0.0, 0.0),
            epsilon = TEST_EPSILON
        );

        // --- Tailwind ---
        let tailwind = Vector3::new(wind_speed, 0.0, 0.0); // Wind towards +X
        let result_tail = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &tailwind,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        // Vrel_body = Vair_body - Vwind_body = [100,0,0] - [20,0,0] = [80,0,0]
        assert_relative_eq!(result_tail.true_airspeed, 80.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result_tail.alpha, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result_tail.beta, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result_tail.relative_velocity_body,
            Vector3::new(80.0, 0.0, 0.0),
            epsilon = TEST_EPSILON
        );

        // --- Crosswind (from Right) ---
        let crosswind_right = Vector3::new(0.0, wind_speed, 0.0); // Wind towards +Y (from aircraft's left)
        let result_cross_right = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &crosswind_right,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        // Vrel_body = Vair_body - Vwind_body = [100,0,0] - [0,20,0] = [100, -20, 0]
        let expected_airspeed_cr = (100.0f64.powi(2) + (-20.0f64).powi(2)).sqrt();
        let expected_beta_cr = (-20.0 / expected_airspeed_cr).asin(); // Should be negative beta
        assert_relative_eq!(
            result_cross_right.true_airspeed,
            expected_airspeed_cr,
            epsilon = TEST_EPSILON
        );
        assert_relative_eq!(result_cross_right.alpha, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result_cross_right.beta,
            expected_beta_cr,
            epsilon = TEST_EPSILON
        );
        assert_relative_eq!(
            result_cross_right.relative_velocity_body,
            Vector3::new(100.0, -20.0, 0.0),
            epsilon = TEST_EPSILON
        );

        // --- Crosswind (from Left) ---
        let crosswind_left = Vector3::new(0.0, -wind_speed, 0.0); // Wind towards -Y (from aircraft's right)
        let result_cross_left = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &crosswind_left,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        // Vrel_body = Vair_body - Vwind_body = [100,0,0] - [0,-20,0] = [100, 20, 0]
        let expected_airspeed_cl = (100.0f64.powi(2) + 20.0f64.powi(2)).sqrt();
        let expected_beta_cl = (20.0 / expected_airspeed_cl).asin(); // Should be positive beta
        assert_relative_eq!(
            result_cross_left.true_airspeed,
            expected_airspeed_cl,
            epsilon = TEST_EPSILON
        );
        assert_relative_eq!(result_cross_left.alpha, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result_cross_left.beta,
            expected_beta_cl,
            epsilon = TEST_EPSILON
        );
        assert_relative_eq!(
            result_cross_left.relative_velocity_body,
            Vector3::new(100.0, 20.0, 0.0),
            epsilon = TEST_EPSILON
        );
    }

    #[test]
    fn test_density_and_dynamic_pressure() {
        // Scenario: Check density passthrough and dynamic pressure calculation at different densities
        let speed = 100.0;
        let velocity_inertial = Vector3::new(speed, 0.0, 0.0);
        let attitude = UnitQuaternion::identity();
        let wind_inertial = Vector3::zeros();

        let test_densities = vec![
            1.225,  // Sea level
            1.112,  // ~1000m
            0.7364, // ~5000m
            0.4135, // ~10000m
        ];

        for density in test_densities {
            let result = calculate_air_data(
                &velocity_inertial,
                &attitude,
                &wind_inertial,
                density,
                MIN_AIRSPEED_THRESHOLD,
            );

            assert_relative_eq!(result.density, density, epsilon = TEST_EPSILON);
            let expected_q = 0.5 * density * speed * speed;
            assert_relative_eq!(result.dynamic_pressure, expected_q, epsilon = TEST_EPSILON);
            // Airspeed, alpha, beta should be unaffected by density
            assert_relative_eq!(result.true_airspeed, speed, epsilon = TEST_EPSILON);
            assert_relative_eq!(result.alpha, 0.0, epsilon = TEST_EPSILON);
            assert_relative_eq!(result.beta, 0.0, epsilon = TEST_EPSILON);
        }
    }

    #[test]
    fn test_low_airspeed_threshold() {
        let density = STD_DENSITY;
        let attitude = UnitQuaternion::identity();
        let wind_inertial = Vector3::zeros();

        // Case 1: Airspeed exactly zero (below threshold)
        let vel_zero = Vector3::zeros();
        let result_zero = calculate_air_data(
            &vel_zero,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        assert_relative_eq!(result_zero.true_airspeed, 0.0, epsilon = TEST_EPSILON);
        assert_relative_eq!(result_zero.alpha, 0.0, epsilon = TEST_EPSILON); // Should be zero
        assert_relative_eq!(result_zero.beta, 0.0, epsilon = TEST_EPSILON); // Should be zero

        // Case 2: Airspeed slightly below threshold (e.g., 0.4)
        let vel_below = Vector3::new(MIN_AIRSPEED_THRESHOLD * 0.8, 0.0, 0.0);
        let result_below = calculate_air_data(
            &vel_below,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        assert_relative_eq!(
            result_below.true_airspeed,
            MIN_AIRSPEED_THRESHOLD * 0.8,
            epsilon = TEST_EPSILON
        );
        assert_relative_eq!(result_below.alpha, 0.0, epsilon = TEST_EPSILON); // Should be zero
        assert_relative_eq!(result_below.beta, 0.0, epsilon = TEST_EPSILON); // Should be zero

        // Case 3: Airspeed slightly above threshold (e.g., 0.6) with non-zero alpha/beta potential
        let speed_above = MIN_AIRSPEED_THRESHOLD * 1.2;
        let angle_rad = 10.0 * PI / 180.0;
        // Velocity vector angled up (positive alpha) and right (positive beta)
        let vx = speed_above * angle_rad.cos() * angle_rad.cos(); // Simplified components
        let vy = speed_above * angle_rad.sin();
        let vz = speed_above * angle_rad.cos() * angle_rad.sin();
        let vel_above = Vector3::new(vx, vy, vz);
        // Recalculate actual speed and angles for verification
        let actual_speed_above = vel_above.norm();
        let actual_alpha = (vz).atan2(vx);
        let actual_beta = (vy / actual_speed_above).clamp(-1.0, 1.0).asin();

        let result_above = calculate_air_data(
            &vel_above,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        assert!(actual_speed_above > MIN_AIRSPEED_THRESHOLD); // Sanity check test setup
        assert_relative_eq!(
            result_above.true_airspeed,
            actual_speed_above,
            epsilon = TEST_EPSILON
        );
        // Alpha/beta should now be calculated (and potentially clamped)
        assert_relative_eq!(
            result_above.alpha,
            actual_alpha.clamp(-30.0 * PI / 180.0, 30.0 * PI / 180.0),
            epsilon = TEST_EPSILON
        );
        assert_relative_eq!(
            result_above.beta,
            actual_beta.clamp(-30.0 * PI / 180.0, 30.0 * PI / 180.0),
            epsilon = TEST_EPSILON
        );
    }

    #[test]
    fn test_non_identity_attitude() {
        // Scenario: Aircraft pitched up 10 degrees, flying level relative to world, no wind.
        let pitch_deg = 10.0;
        let pitch_rad = pitch_deg * PI / 180.0;
        let speed = 100.0;

        let velocity_inertial = Vector3::new(speed, 0.0, 0.0); // Level flight in world frame
        let attitude = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), pitch_rad); // Pitched up (+Y is right, rotation around Y is pitch)
        let wind_inertial = Vector3::zeros();
        let density = STD_DENSITY;

        // Expected body velocity: rotate inertial velocity by inverse attitude
        // Inverse attitude is pitch down (-10 deg)
        // let expected_velocity_body = attitude.inverse() * velocity_inertial;
        // Rotation matrix for -pitch_rad around Y: [[cos, 0, sin], [0, 1, 0], [-sin, 0, cos]]
        // [Vx, Vy, Vz]_body = [speed*cos(-pitch), 0, speed*-sin(-pitch)] = [speed*cos(pitch), 0, speed*sin(pitch)]
        let expected_vx_body = speed * pitch_rad.cos();
        let expected_vz_body = speed * pitch_rad.sin();
        let expected_vel_b = Vector3::new(expected_vx_body, 0.0, expected_vz_body);

        // Expected alpha: atan2(Vz_body, Vx_body) = atan2(sin(pitch), cos(pitch)) = pitch_rad
        let expected_alpha = pitch_rad;
        let expected_beta = 0.0; // No sideslip expected

        let result = calculate_air_data(
            &velocity_inertial,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );

        assert_relative_eq!(result.true_airspeed, speed, epsilon = TEST_EPSILON); // Airspeed magnitude remains the same
        assert_relative_eq!(result.alpha, expected_alpha, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.beta, expected_beta, epsilon = TEST_EPSILON);
        assert_relative_eq!(result.density, density, epsilon = TEST_EPSILON);
        assert_relative_eq!(
            result.relative_velocity_body,
            expected_vel_b,
            epsilon = TEST_EPSILON
        );
    }

    #[test]
    fn test_alpha_beta_clamping() {
        // Scenario: Input velocity results in angles outside the +/- 30 deg clamp range.
        let speed = 100.0;
        let density = STD_DENSITY;
        let attitude = UnitQuaternion::identity();
        let wind_inertial = Vector3::zeros();
        let clamp_limit_rad = 30.0 * PI / 180.0;

        // --- High Alpha Test ---
        let high_alpha_deg = 45.0; // Exceeds 30 deg clamp
        let high_alpha_rad = high_alpha_deg * PI / 180.0;
        let vx_body_ha = speed * high_alpha_rad.cos();
        let vz_body_ha = speed * high_alpha_rad.sin(); // Positive Vz for positive alpha
        let vel_inertial_ha = Vector3::new(vx_body_ha, 0.0, vz_body_ha);

        let result_ha = calculate_air_data(
            &vel_inertial_ha,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        assert!(high_alpha_rad > clamp_limit_rad); // Verify test setup
        assert_relative_eq!(result_ha.alpha, clamp_limit_rad, epsilon = TEST_EPSILON); // Should be clamped
        assert_relative_eq!(result_ha.beta, 0.0, epsilon = TEST_EPSILON);

        // --- High Beta Test ---
        let high_beta_deg = 45.0; // Exceeds 30 deg clamp
        let high_beta_rad = high_beta_deg * PI / 180.0;
        let vx_body_hb = speed * high_beta_rad.cos();
        let vy_body_hb = speed * high_beta_rad.sin(); // Positive Vy for positive beta
        let vel_inertial_hb = Vector3::new(vx_body_hb, vy_body_hb, 0.0);
        // Ensure speed is still roughly correct
        assert_relative_eq!(vel_inertial_hb.norm(), speed, epsilon = TEST_EPSILON);

        let result_hb = calculate_air_data(
            &vel_inertial_hb,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        // Actual beta before clamp: asin(vy/speed) = asin(sin(45)) = 45 deg
        assert!(high_beta_rad > clamp_limit_rad); // Verify test setup
        assert_relative_eq!(result_hb.beta, clamp_limit_rad, epsilon = TEST_EPSILON); // Should be clamped
        assert_relative_eq!(result_hb.alpha, 0.0, epsilon = TEST_EPSILON);

        // --- Negative High Alpha Test ---
        let neg_high_alpha_deg = -45.0; // Exceeds -30 deg clamp
        let neg_high_alpha_rad = neg_high_alpha_deg * PI / 180.0;
        let vx_body_nha = speed * neg_high_alpha_rad.cos(); // Still positive
        let vz_body_nha = speed * neg_high_alpha_rad.sin(); // Negative Vz for negative alpha
        let vel_inertial_nha = Vector3::new(vx_body_nha, 0.0, vz_body_nha);

        let result_nha = calculate_air_data(
            &vel_inertial_nha,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        assert!(neg_high_alpha_rad < -clamp_limit_rad); // Verify test setup
        assert_relative_eq!(result_nha.alpha, -clamp_limit_rad, epsilon = TEST_EPSILON); // Should be clamped
        assert_relative_eq!(result_nha.beta, 0.0, epsilon = TEST_EPSILON);

        // --- Negative High Beta Test ---
        let neg_high_beta_deg = -45.0; // Exceeds -30 deg clamp
        let neg_high_beta_rad = neg_high_beta_deg * PI / 180.0;
        let vx_body_nhb = speed * neg_high_beta_rad.cos(); // Still positive
        let vy_body_nhb = speed * neg_high_beta_rad.sin(); // Negative Vy for negative beta
        let vel_inertial_nhb = Vector3::new(vx_body_nhb, vy_body_nhb, 0.0);
        // Ensure speed is still roughly correct
        assert_relative_eq!(vel_inertial_nhb.norm(), speed, epsilon = TEST_EPSILON);

        let result_nhb = calculate_air_data(
            &vel_inertial_nhb,
            &attitude,
            &wind_inertial,
            density,
            MIN_AIRSPEED_THRESHOLD,
        );
        // Actual beta before clamp: asin(vy/speed) = asin(sin(-45)) = -45 deg
        assert!(neg_high_beta_rad < -clamp_limit_rad); // Verify test setup
        assert_relative_eq!(result_nhb.beta, -clamp_limit_rad, epsilon = TEST_EPSILON); // Should be clamped
        assert_relative_eq!(result_nhb.alpha, 0.0, epsilon = TEST_EPSILON);
    }
}
