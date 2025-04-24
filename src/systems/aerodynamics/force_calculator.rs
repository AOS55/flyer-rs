use bevy::prelude::*;
use nalgebra::Vector3;
use std::f64::consts::PI;

use crate::components::{
    AirData, AircraftAeroCoefficients, AircraftControlSurfaces, AircraftGeometry, Force,
    ForceCategory, FullAircraftConfig, Moment, PhysicsComponent, ReferenceFrame, SpatialComponent,
};
use crate::resources::AerodynamicsConfig;

// Assuming AirDataValues is defined elsewhere (e.g., air_data.rs or calculate.rs)
use crate::systems::AirDataValues;

// --- Pure Calculation Logic ---

/// Calculates aerodynamic forces and moments in the BODY frame based on aircraft state.
/// This is the "pure function" part.
///
/// # Arguments
/// * `geometry` - Aircraft geometric properties.
/// * `coeffs` - Aircraft aerodynamic coefficients.
/// * `air_data` - Calculated air data values (airspeed, alpha, beta, q, etc.).
/// * `angular_velocity_body` - Rotational rates in the body frame (p, q, r).
/// * `controls` - Current control surface deflections.
///
/// # Returns
/// A tuple containing: `(body_forces: Vector3<f64>, body_moments: Vector3<f64>)`
pub fn calculate_aerodynamic_forces_moments(
    geometry: &AircraftGeometry,
    coeffs: &AircraftAeroCoefficients,
    air_data: &AirDataValues,
    angular_velocity_body: &Vector3<f64>,
    controls: &AircraftControlSurfaces,
) -> (Vector3<f64>, Vector3<f64>) {
    // Early exit if no dynamic pressure or very low airspeed
    if air_data.dynamic_pressure <= 1e-6 || air_data.true_airspeed <= 0.1 {
        return (Vector3::zeros(), Vector3::zeros());
    }

    // --- Replicate logic from AersoAdapter::compute_forces ---
    // Get necessary values from air_data struct
    let alpha = air_data.alpha;
    let beta = air_data.beta;
    let q_dyn = air_data.dynamic_pressure; // Dynamic pressure 'q'

    // Clamp angles and rates to valid ranges (using example values from original code)
    let alpha = alpha.clamp(-10.0 * PI / 180.0, 40.0 * PI / 180.0);
    let beta = beta.clamp(-20.0 * PI / 180.0, 20.0 * PI / 180.0);
    let p = angular_velocity_body
        .x
        .clamp(-100.0 * PI / 180.0, 100.0 * PI / 180.0);
    let q = angular_velocity_body
        .y
        .clamp(-50.0 * PI / 180.0, 50.0 * PI / 180.0);
    let r = angular_velocity_body
        .z
        .clamp(-50.0 * PI / 180.0, 50.0 * PI / 180.0);

    // Calculate non-dimensional rates (p_hat, q_hat, r_hat)
    let airspeed = air_data.true_airspeed;
    let span = geometry.wing_span;
    let mac = geometry.mac;
    let v_denom = 2.0 * airspeed + 1e-9; // Add epsilon for stability at low speed
    let p_hat = (span / v_denom) * p;
    let q_hat = (mac / v_denom) * q;
    let r_hat = (span / v_denom) * r;

    // --- Calculate Aerodynamic Coefficients (CD, CY, CL, Cl, Cm, Cn) ---
    // (This section directly copies the coefficient calculations from the original AersoAdapter)
    let c_d = coeffs.drag.c_d_0
        + (coeffs.drag.c_d_alpha * alpha)
        + (coeffs.drag.c_d_alpha_q * alpha * q_hat)
        + (coeffs.drag.c_d_alpha_deltae * alpha * controls.elevator)
        + (coeffs.drag.c_d_alpha2 * alpha.powi(2))
        + (coeffs.drag.c_d_alpha2_q * q_hat * alpha.powi(2))
        + (coeffs.drag.c_d_alpha2_deltae * controls.elevator * alpha.powi(2))
        + (coeffs.drag.c_d_alpha3 * alpha.powi(3))
        + (coeffs.drag.c_d_alpha3_q * q_hat * alpha.powi(3))
        + (coeffs.drag.c_d_alpha4 * alpha.powi(4));

    let c_y = coeffs.side_force.c_y_beta * beta
        + (coeffs.side_force.c_y_p * p_hat)
        + (coeffs.side_force.c_y_r * r_hat)
        + (coeffs.side_force.c_y_deltaa * controls.aileron)
        + (coeffs.side_force.c_y_deltar * controls.rudder);

    let c_l = coeffs.lift.c_l_0
        + (coeffs.lift.c_l_alpha * alpha)
        + (coeffs.lift.c_l_q * q_hat)
        + (coeffs.lift.c_l_deltae * controls.elevator)
        + (coeffs.lift.c_l_alpha_q * alpha * q_hat)
        + (coeffs.lift.c_l_alpha2 * alpha.powi(2))
        + (coeffs.lift.c_l_alpha3 * alpha.powi(3))
        + (coeffs.lift.c_l_alpha4 * alpha.powi(4));

    let c_l_roll = coeffs.roll.c_l_beta * beta // Roll moment coefficient 'Cl'
        + (coeffs.roll.c_l_p * p_hat)
        + (coeffs.roll.c_l_r * r_hat)
        + (coeffs.roll.c_l_deltaa * controls.aileron)
        + (coeffs.roll.c_l_deltar * controls.rudder);

    let c_m = coeffs.pitch.c_m_0             // Pitch moment coefficient 'Cm'
        + (coeffs.pitch.c_m_alpha * alpha)
        + (coeffs.pitch.c_m_q * q_hat)
        + (coeffs.pitch.c_m_deltae * controls.elevator)
        + (coeffs.pitch.c_m_alpha_q * alpha * q_hat)
        + (coeffs.pitch.c_m_alpha2_q * q_hat * alpha.powi(2))
        + (coeffs.pitch.c_m_alpha2_deltae * controls.elevator * alpha.powi(2))
        + (coeffs.pitch.c_m_alpha3_q * q_hat * alpha.powi(3))
        + (coeffs.pitch.c_m_alpha3_deltae * controls.elevator * alpha.powi(3))
        + (coeffs.pitch.c_m_alpha4 * alpha.powi(4));

    let c_n = coeffs.yaw.c_n_beta * beta     // Yaw moment coefficient 'Cn'
        + (coeffs.yaw.c_n_p * p_hat)
        + (coeffs.yaw.c_n_r * r_hat)
        + (coeffs.yaw.c_n_deltaa * controls.aileron)
        + (coeffs.yaw.c_n_deltar * controls.rudder)
        + (coeffs.yaw.c_n_beta2 * beta.powi(2))
        + (coeffs.yaw.c_n_beta3 * beta.powi(3));

    // --- Calculate Forces (Body Frame) ---
    // Standard aero axes convention: Fx (drag is neg), Fy (sideforce), Fz (lift is neg)
    let forces_body = Vector3::new(
        -q_dyn * geometry.wing_area * c_d, // Drag opposes positive X
        q_dyn * geometry.wing_area * c_y,  // Sideforce along positive Y
        -q_dyn * geometry.wing_area * c_l, // Lift opposes positive Z (points up)
    );

    // --- Calculate Moments (Body Frame) ---
    // Standard aero axes convention: L (roll), M (pitch), N (yaw)
    let moments_body = Vector3::new(
        q_dyn * geometry.wing_area * geometry.wing_span * c_l_roll, // Roll Moment (L) about X axis
        q_dyn * geometry.wing_area * geometry.mac * c_m,            // Pitch Moment (M) about Y axis
        q_dyn * geometry.wing_area * geometry.wing_span * c_n,      // Yaw Moment (N) about Z axis
    );

    (forces_body, moments_body)
}

/// System for calculating aerodynamic forces and moments acting on aircraft.
/// Queries components, calls the pure calculation function, updates PhysicsComponent.
pub fn aero_force_system(
    mut aircraft: Query<(
        &AircraftControlSurfaces,
        &AirData, // Query the component to get input values
        &SpatialComponent,
        &mut PhysicsComponent, // Need mutable access to add forces/moments
        &FullAircraftConfig,   // Contains geometry and coefficients
    )>,
    aero_config: Res<AerodynamicsConfig>, // Keep config for threshold check
) {
    for (controls, air_data_comp, spatial, mut physics, config) in aircraft.iter_mut() {
        // 1. Perform pre-checks (e.g., airspeed threshold)
        if air_data_comp.true_airspeed < aero_config.min_airspeed_threshold {
            // If skipping calculation, ensure any previous aero forces are cleared
            // to prevent stale forces from persisting.
            physics
                .forces
                .retain(|f| f.category != ForceCategory::Aerodynamic);
            physics
                .moments
                .retain(|m| m.category != ForceCategory::Aerodynamic);
            continue; // Skip this entity if below threshold
        }

        // 2. Prepare inputs for the pure calculation function
        // Create the simple AirDataValues struct from the component fields
        // Assumes air_data_system ran before this system in the Bevy schedule
        let air_data_values = AirDataValues {
            true_airspeed: air_data_comp.true_airspeed,
            alpha: air_data_comp.alpha,
            beta: air_data_comp.beta,
            density: air_data_comp.density,
            dynamic_pressure: air_data_comp.dynamic_pressure,
            relative_velocity_body: air_data_comp.relative_velocity,
        };

        // 3. Call the pure calculation function
        let (forces_body, moments_body) = calculate_aerodynamic_forces_moments(
            &config.geometry,          // Pass geometry ref from config
            &config.aero_coef,         // Pass aero coefficients ref from config
            &air_data_values,          // Pass the prepared air data values struct
            &spatial.angular_velocity, // Pass body-frame angular velocity from spatial
            controls,                  // Pass controls state ref
        );

        // 4. Update the PhysicsComponent
        // Clear existing aerodynamic forces/moments first
        physics
            .forces
            .retain(|f| f.category != ForceCategory::Aerodynamic);
        physics
            .moments
            .retain(|m| m.category != ForceCategory::Aerodynamic);

        // Add the newly calculated force/moment if they are significant
        // Use a small threshold to avoid adding negligible floating point noise
        if forces_body.norm_squared() > 1e-9 {
            physics.add_force(Force {
                vector: forces_body,
                point: None, // Aerodynamic forces typically applied at Aerodynamic Center, but often simplified to CG
                frame: ReferenceFrame::Body, // The function calculates in Body frame
                category: ForceCategory::Aerodynamic,
            });
        }
        if moments_body.norm_squared() > 1e-9 {
            physics.add_moment(Moment {
                vector: moments_body,
                frame: ReferenceFrame::Body, // The function calculates in Body frame
                category: ForceCategory::Aerodynamic,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Imports calculate_aerodynamic_forces_moments
    use crate::components::{AircraftAeroCoefficients, AircraftControlSurfaces, AircraftGeometry};
    // Assuming AirDataValues is defined elsewhere (e.g., air_data.rs or calculate.rs)
    // You might need to adjust this path if AirDataValues is in a different location
    use crate::systems::aerodynamics::air_data::AirDataValues; // Corrected potential path

    use approx::assert_relative_eq;
    use nalgebra::Vector3;
    use std::f64::consts::PI;

    // --- Constants for Tests ---
    const TEST_EPSILON: f64 = 1e-6; // Tolerance for floating point comparisons
    const STD_DENSITY: f64 = 1.225;

    // --- Helper Functions for Test Setup ---

    fn default_geometry() -> AircraftGeometry {
        AircraftGeometry {
            wing_area: 39.0, // m² (Twin Otter based)
            wing_span: 19.8, // m
            mac: 1.98,       // m Mean Aerodynamic Chord
        }
    }

    fn default_coeffs() -> AircraftAeroCoefficients {
        // Using Twin Otter coefficients as a base for realistic values
        AircraftAeroCoefficients::twin_otter()
    }

    fn default_controls() -> AircraftControlSurfaces {
        AircraftControlSurfaces::default() // All surfaces neutral
    }

    // Helper to create AirDataValues struct for tests
    fn create_air_data(speed: f64, alpha_rad: f64, beta_rad: f64, density: f64) -> AirDataValues {
        let dynamic_pressure = 0.5 * density * speed.max(0.0).powi(2); // Ensure speed isn't negative for q calc
                                                                       // Approximate relative velocity based on alpha/beta.
                                                                       // Note: function uses norm of relative_velocity_body if needed,
                                                                       // but airspeed is passed directly. Body velocity only needed if function used it.
        let vx = speed * alpha_rad.cos() * beta_rad.cos();
        let vy = speed * beta_rad.sin();
        let vz = speed * alpha_rad.sin() * beta_rad.cos(); // Note: alpha defined atan2(vz, vx)

        AirDataValues {
            true_airspeed: speed,
            alpha: alpha_rad,
            beta: beta_rad,
            density,
            dynamic_pressure,
            // This field isn't directly used by calculate_aerodynamic_forces_moments,
            // but we fill it for completeness based on the other inputs.
            relative_velocity_body: Vector3::new(vx, vy, vz),
        }
    }

    // --- Test Cases ---

    #[test]
    fn test_zero_airspeed_returns_zero_forces() {
        let geometry = default_geometry();
        let coeffs = default_coeffs();
        // Use speed slightly below internal threshold check for robustness
        let air_data = create_air_data(0.05, 0.0, 0.0, STD_DENSITY);
        let angular_velocity_body = Vector3::zeros();
        let controls = default_controls();

        // Call the function being tested
        let (forces, moments) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data, // Airspeed/q should trigger early exit
            &angular_velocity_body,
            &controls,
        );

        // Assertions
        assert_relative_eq!(forces, Vector3::zeros(), epsilon = TEST_EPSILON);
        assert_relative_eq!(moments, Vector3::zeros(), epsilon = TEST_EPSILON);

        // Test strictly zero case too
        let air_data_zero = create_air_data(0.0, 0.0, 0.0, STD_DENSITY);
        let (forces_z, moments_z) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data_zero,
            &angular_velocity_body,
            &controls,
        );
        assert_relative_eq!(forces_z, Vector3::zeros(), epsilon = TEST_EPSILON);
        assert_relative_eq!(moments_z, Vector3::zeros(), epsilon = TEST_EPSILON);
    }

    #[test]
    fn test_level_flight_positive_alpha() {
        // Scenario: Steady level flight, small positive alpha, no sideslip, no rates, no controls
        let speed = 50.0;
        let alpha_deg = 3.0;
        let alpha_rad = alpha_deg * PI / 180.0;
        let geometry = default_geometry();
        let coeffs = default_coeffs();
        let air_data = create_air_data(speed, alpha_rad, 0.0, STD_DENSITY);
        let angular_velocity_body = Vector3::zeros();
        let controls = default_controls(); // Elevator, aileron, rudder = 0

        // --- Calculate Expected Values using FULL formulas from the function ---
        let q_dyn = air_data.dynamic_pressure;
        let s_area = geometry.wing_area;
        let mac = geometry.mac;
        let span = geometry.wing_span;

        // Clamp alpha as done inside the function
        let alpha = alpha_rad.clamp(-10.0 * PI / 180.0, 40.0 * PI / 180.0);
        // Beta and rates are zero for this test
        let beta = 0.0;

        // Calculate expected coefficients fully
        let expected_cd = coeffs.drag.c_d_0
            + (coeffs.drag.c_d_alpha * alpha)
            // + (coeffs.drag.c_d_alpha_q * alpha * q_hat) // q_hat=0
            // + (coeffs.drag.c_d_alpha_deltae * alpha * controls.elevator) // elevator=0
            + (coeffs.drag.c_d_alpha2 * alpha.powi(2))
            // + (coeffs.drag.c_d_alpha2_q * q_hat * alpha.powi(2)) // q_hat=0
            // + (coeffs.drag.c_d_alpha2_deltae * controls.elevator * alpha.powi(2)) // elevator=0
            + (coeffs.drag.c_d_alpha3 * alpha.powi(3))
            // + (coeffs.drag.c_d_alpha3_q * q_hat * alpha.powi(3)) // q_hat=0
            + (coeffs.drag.c_d_alpha4 * alpha.powi(4));

        let expected_cy = coeffs.side_force.c_y_beta * beta // beta=0
            // + (coeffs.side_force.c_y_p * p_hat) // p_hat=0
            // + (coeffs.side_force.c_y_r * r_hat) // r_hat=0
            // + (coeffs.side_force.c_y_deltaa * controls.aileron) // aileron=0
            // + (coeffs.side_force.c_y_deltar * controls.rudder); // rudder=0
             + 0.0; // Explicitly zero for this case

        let expected_cl = coeffs.lift.c_l_0
            + (coeffs.lift.c_l_alpha * alpha)
            // + (coeffs.lift.c_l_q * q_hat) // q_hat=0
            // + (coeffs.lift.c_l_deltae * controls.elevator) // elevator=0
            // + (coeffs.lift.c_l_alpha_q * alpha * q_hat) // q_hat=0
            + (coeffs.lift.c_l_alpha2 * alpha.powi(2))
            + (coeffs.lift.c_l_alpha3 * alpha.powi(3))
            + (coeffs.lift.c_l_alpha4 * alpha.powi(4));

        let expected_cl_roll = coeffs.roll.c_l_beta * beta // beta=0
            // + (coeffs.roll.c_l_p * p_hat) // p_hat=0
            // + (coeffs.roll.c_l_r * r_hat) // r_hat=0
            // + (coeffs.roll.c_l_deltaa * controls.aileron) // aileron=0
            // + (coeffs.roll.c_l_deltar * controls.rudder); // rudder=0
            + 0.0; // Explicitly zero

        let expected_cm = coeffs.pitch.c_m_0
            + (coeffs.pitch.c_m_alpha * alpha)
            // + (coeffs.pitch.c_m_q * q_hat) // q_hat=0
            // + (coeffs.pitch.c_m_deltae * controls.elevator) // elevator=0
            // + (coeffs.pitch.c_m_alpha_q * alpha * q_hat) // q_hat=0
            // + (coeffs.pitch.c_m_alpha2_q * q_hat * alpha.powi(2)) // q_hat=0
            // + (coeffs.pitch.c_m_alpha2_deltae * controls.elevator * alpha.powi(2)) // elevator=0
            // + (coeffs.pitch.c_m_alpha3_q * q_hat * alpha.powi(3)) // q_hat=0
            // + (coeffs.pitch.c_m_alpha3_deltae * controls.elevator * alpha.powi(3)) // elevator=0
            + (coeffs.pitch.c_m_alpha4 * alpha.powi(4));

        let expected_cn = coeffs.yaw.c_n_beta * beta // beta=0
            // + (coeffs.yaw.c_n_p * p_hat) // p_hat=0
            // + (coeffs.yaw.c_n_r * r_hat) // r_hat=0
            // + (coeffs.yaw.c_n_deltaa * controls.aileron) // aileron=0
            // + (coeffs.yaw.c_n_deltar * controls.rudder) // rudder=0
            // + (coeffs.yaw.c_n_beta2 * beta.powi(2)) // beta=0
            // + (coeffs.yaw.c_n_beta3 * beta.powi(3)); // beta=0
             + 0.0; // Explicitly zero

        // Expected Forces/Moments using full coefficients
        let expected_forces = Vector3::new(
            -q_dyn * s_area * expected_cd, // Drag opposes +X
            q_dyn * s_area * expected_cy,  // Sideforce along +Y
            -q_dyn * s_area * expected_cl, // Lift opposes +Z
        );
        let expected_moments = Vector3::new(
            q_dyn * s_area * span * expected_cl_roll, // Roll Moment (L) about X
            q_dyn * s_area * mac * expected_cm,       // Pitch Moment (M) about Y
            q_dyn * s_area * span * expected_cn,      // Yaw Moment (N) about Z
        );

        // --- Call the function being tested ---
        let (forces, moments) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &angular_velocity_body,
            &controls,
        );

        // --- Assertions ---
        // Assert against the fully calculated expected values with tight epsilon
        assert_relative_eq!(forces.x, expected_forces.x, epsilon = TEST_EPSILON);
        assert_relative_eq!(forces.y, expected_forces.y, epsilon = TEST_EPSILON);
        assert_relative_eq!(forces.z, expected_forces.z, epsilon = TEST_EPSILON);

        assert_relative_eq!(moments.x, expected_moments.x, epsilon = TEST_EPSILON);
        assert_relative_eq!(moments.y, expected_moments.y, epsilon = TEST_EPSILON);
        assert_relative_eq!(moments.z, expected_moments.z, epsilon = TEST_EPSILON);
    }

    #[test]
    fn test_alpha_sweep_stall() {
        let speed = 50.0;
        let geometry = default_geometry();
        let coeffs = default_coeffs();
        let air_data_base = create_air_data(speed, 0.0, 0.0, STD_DENSITY); // Base q
        let angular_velocity_body = Vector3::zeros();
        let controls = default_controls();
        let q_dyn = air_data_base.dynamic_pressure;
        let s_area = geometry.wing_area;

        let mut cl_values = Vec::new();
        let test_angles_deg = vec![0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0]; // Sweep through alpha

        for angle_deg in test_angles_deg {
            let alpha_rad = angle_deg * PI / 180.0;
            let air_data = create_air_data(speed, alpha_rad, 0.0, STD_DENSITY); // Update alpha

            let (forces, _moments) = calculate_aerodynamic_forces_moments(
                &geometry,
                &coeffs,
                &air_data,
                &angular_velocity_body,
                &controls,
            );

            // Calculate CL from the result: CL = -Fz / (q * S)
            let cl = -forces.z / (q_dyn * s_area + 1e-9); // Add epsilon to avoid div by zero if q=0
            cl_values.push((angle_deg, cl));
            // println!("Angle: {:.1}°, CL: {:.4}", angle_deg, cl);
        }

        // Basic Stall Check: Find max CL and ensure the last CL is lower or angle didn't increase past max.
        let mut max_cl = f64::NEG_INFINITY;
        let mut angle_at_max_cl = -1.0; // Initialize to value below test range
        for (angle, cl) in &cl_values {
            if *cl > max_cl {
                // Allow for slight decreases due to floating point before true peak
                if *cl >= max_cl - TEST_EPSILON * 10.0 {
                    max_cl = *cl;
                    angle_at_max_cl = *angle;
                }
            }
        }
        let last_cl = cl_values.last().unwrap().1;
        let last_angle = cl_values.last().unwrap().0;

        assert!(max_cl > 0.0, "Max CL should be positive.");
        // Check if CL dropped after the peak OR if we hit the max angle tested at the peak
        assert!(
            last_cl < max_cl + TEST_EPSILON || (last_angle - angle_at_max_cl).abs() < TEST_EPSILON,
            "CL ({:.4} @ {:.1}°) should decrease or plateau after the peak CL ({:.4} @ {:.1}°)",
            last_cl,
            last_angle,
            max_cl,
            angle_at_max_cl
        );
        // Check initial slope is positive
        assert!(
            cl_values.len() >= 2 && cl_values[1].1 > cl_values[0].1 - TEST_EPSILON,
            "CL should initially increase with AoA."
        );
    }

    #[test]
    fn test_elevator_pitch_moment() {
        let speed = 50.0;
        let elevator_deflection = -0.2; // Negative deflection = trailing edge down = nose down pitch moment
        let geometry = default_geometry();
        let coeffs = default_coeffs();
        // Neutral flight condition otherwise
        let air_data = create_air_data(speed, 0.0, 0.0, STD_DENSITY); // alpha=0
        let angular_velocity_body = Vector3::zeros(); // q=0
        let mut controls = default_controls();
        controls.elevator = elevator_deflection;

        let q_dyn = air_data.dynamic_pressure;
        let s_area = geometry.wing_area;
        let mac = geometry.mac;

        // --- Calculate Expected Values using FULL formulas (simplified for alpha=0, q=0) ---
        let alpha: f64 = 0.0;
        let expected_cm = coeffs.pitch.c_m_0 // Base term
            // + (coeffs.pitch.c_m_alpha * alpha) // alpha=0
            // + (coeffs.pitch.c_m_q * q_hat) // q_hat=0
            + (coeffs.pitch.c_m_deltae * controls.elevator) // Elevator term
            // + (coeffs.pitch.c_m_alpha_q * alpha * q_hat) // alpha=0 or q_hat=0
            // + (coeffs.pitch.c_m_alpha2_q * q_hat * alpha.powi(2)) // alpha=0 or q_hat=0
            + (coeffs.pitch.c_m_alpha2_deltae * controls.elevator * alpha.powi(2)) // alpha=0
            // + (coeffs.pitch.c_m_alpha3_q * q_hat * alpha.powi(3)) // alpha=0 or q_hat=0
            + (coeffs.pitch.c_m_alpha3_deltae * controls.elevator * alpha.powi(3)) // alpha=0
            // + (coeffs.pitch.c_m_alpha4 * alpha.powi(4)); // alpha=0
             ; // Simplified to Cm0 + Cm_de * elevator for this case

        let expected_pitch_moment = q_dyn * s_area * mac * expected_cm;

        // --- Call the function ---
        let (_forces, moments) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &angular_velocity_body,
            &controls,
        );

        // --- Assertions ---
        // Check pitch moment (My) against the more accurate expectation
        assert_relative_eq!(moments.y, expected_pitch_moment, epsilon = TEST_EPSILON);

        // Check other moments are near zero (assuming no cross-coupling from elevator alone)
        // Calculate base moments expected from Cm0/Cl0 etc if needed, but should be zero for roll/yaw here.
        assert_relative_eq!(moments.x, 0.0, epsilon = q_dyn * 1e-3); // Roll
        assert_relative_eq!(moments.z, 0.0, epsilon = q_dyn * 1e-3); // Yaw

        // Verify sign (based on calculated expected value)
        assert!(
            moments.y.signum() == expected_pitch_moment.signum(),
            "Pitch moment sign mismatch"
        );
        // Optionally add specific check based on common knowledge IF coeffs match
        if coeffs.pitch.c_m_deltae < 0.0
            && coeffs.pitch.c_m_0.abs() < (coeffs.pitch.c_m_deltae * elevator_deflection).abs()
        {
            assert!(
                moments.y < 0.0,
                "Calculated pitch moment should be negative for neg elevator & neg Cm_de"
            );
        }
    }

    #[test]
    fn test_aileron_roll_moment() {
        let speed = 50.0;
        let aileron_deflection = 0.2; // Positive deflection = right roll command = negative roll moment (L)
        let geometry = default_geometry();
        let coeffs = default_coeffs();
        let air_data = create_air_data(speed, 0.0, 0.0, STD_DENSITY); // alpha=0
        let angular_velocity_body = Vector3::zeros(); // p=0, r=0
        let mut controls = default_controls();
        controls.aileron = aileron_deflection;

        let q_dyn = air_data.dynamic_pressure;
        let s_area = geometry.wing_area;
        let span = geometry.wing_span;
        let mac = geometry.mac; // Needed for pitch moment check

        // --- Calculate Expected Values (simplified for alpha=0, beta=0, p=0, r=0) ---
        let expected_cl_roll = coeffs.roll.c_l_deltaa * aileron_deflection; // Dominated by aileron
        let expected_roll_moment = q_dyn * s_area * span * expected_cl_roll;

        let expected_cn = coeffs.yaw.c_n_deltaa * aileron_deflection; // Adverse yaw
        let expected_adverse_yaw_moment = q_dyn * s_area * span * expected_cn;

        // Pitch moment should only have Cm0 contribution here
        let expected_cm = coeffs.pitch.c_m_0;
        let expected_pitch_moment_base = q_dyn * s_area * mac * expected_cm;

        // --- Call the function ---
        let (_forces, moments) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &angular_velocity_body,
            &controls,
        );

        // --- Assertions ---
        // Check roll moment (Mx)
        assert_relative_eq!(moments.x, expected_roll_moment, epsilon = TEST_EPSILON);
        // Check yaw moment (Mz) for adverse yaw
        assert_relative_eq!(
            moments.z,
            expected_adverse_yaw_moment,
            epsilon = TEST_EPSILON
        );
        // Check pitch moment (My) is close to base Cm0 contribution
        assert_relative_eq!(
            moments.y,
            expected_pitch_moment_base,
            epsilon = expected_pitch_moment_base.abs().max(1.0) * 1e-3
        );

        // Verify signs based on expected coefficients
        if coeffs.roll.c_l_deltaa < 0.0 {
            assert!(
                moments.x < 0.0,
                "Expected negative roll moment for positive aileron & neg Cl_da"
            );
        } else {
            assert!(
                moments.x > 0.0,
                "Expected positive roll moment for positive aileron & pos Cl_da"
            );
        }
        // Verify adverse yaw sign (depends on coefficient sign, Twin Otter cn_da is positive -> positive N for positive aileron)
        if coeffs.yaw.c_n_deltaa > 0.0 {
            assert!(
                moments.z > 0.0,
                "Expected positive adverse yaw moment for positive aileron & pos Cn_da"
            );
        } else {
            assert!(
                moments.z < 0.0,
                "Expected negative adverse yaw moment for positive aileron & neg Cn_da"
            );
        }
    }

    #[test]
    fn test_rudder_yaw_moment() {
        let speed = 50.0;
        let rudder_deflection = 0.2; // Positive deflection = trailing edge left = yaw right = positive yaw moment (N)
        let geometry = default_geometry();
        let coeffs = default_coeffs();
        let air_data = create_air_data(speed, 0.0, 0.0, STD_DENSITY); // alpha=0, beta=0
        let angular_velocity_body = Vector3::zeros(); // p=0, r=0
        let mut controls = default_controls();
        controls.rudder = rudder_deflection;

        let q_dyn = air_data.dynamic_pressure;
        let s_area = geometry.wing_area;
        let span = geometry.wing_span;
        let mac = geometry.mac; // Needed for pitch moment check

        // --- Calculate Expected Values (simplified for alpha=0, beta=0, p=0, r=0) ---
        let expected_cn = coeffs.yaw.c_n_deltar * rudder_deflection; // Dominated by rudder
        let expected_yaw_moment = q_dyn * s_area * span * expected_cn;

        let expected_cl_roll = coeffs.roll.c_l_deltar * rudder_deflection; // Rudder-roll coupling
        let expected_rudder_roll_moment = q_dyn * s_area * span * expected_cl_roll;

        // Pitch moment should only have Cm0 contribution here
        let expected_cm = coeffs.pitch.c_m_0;
        let expected_pitch_moment_base = q_dyn * s_area * mac * expected_cm;

        // --- Call the function ---
        let (_forces, moments) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &angular_velocity_body,
            &controls,
        );

        // --- Assertions ---
        // Check yaw moment (Mz)
        assert_relative_eq!(moments.z, expected_yaw_moment, epsilon = TEST_EPSILON);
        // Check roll moment (Mx) for rudder-roll coupling
        assert_relative_eq!(
            moments.x,
            expected_rudder_roll_moment,
            epsilon = TEST_EPSILON
        );
        // Check pitch moment (My) is close to base Cm0 contribution
        assert_relative_eq!(
            moments.y,
            expected_pitch_moment_base,
            epsilon = expected_pitch_moment_base.abs().max(1.0) * 1e-3
        );

        // Verify signs based on expected coefficients
        if coeffs.yaw.c_n_deltar > 0.0 {
            assert!(
                moments.z > 0.0,
                "Expected positive yaw moment for positive rudder & pos Cn_dr"
            );
        } else {
            assert!(
                moments.z < 0.0,
                "Expected negative yaw moment for positive rudder & neg Cn_dr"
            );
        }
        // Verify rudder roll sign (depends on coefficient sign, Twin Otter cl_dr is positive -> positive L for positive rudder)
        if coeffs.roll.c_l_deltar > 0.0 {
            assert!(
                moments.x > 0.0,
                "Expected positive roll moment from positive rudder & pos Cl_dr"
            );
        } else {
            assert!(
                moments.x < 0.0,
                "Expected negative roll moment from positive rudder & neg Cl_dr"
            );
        }
    }

    #[test]
    fn test_pitch_damping() {
        // Scenario: Pitch rate 'q' present, check damping moment 'M'
        let speed = 50.0;
        let alpha_rad = 0.05; // Small positive alpha
        let pitch_rate_q = 0.1; // rad/s (positive pitch rate = nose up)
        let geometry = default_geometry();
        let coeffs = default_coeffs();
        let air_data = create_air_data(speed, alpha_rad, 0.0, STD_DENSITY);
        let angular_velocity_body = Vector3::new(0.0, pitch_rate_q, 0.0); // Only pitch rate
        let controls = default_controls();

        // --- Calculate Expected Moment WITHOUT damping ---
        let (_forces_no_q, moments_no_q) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &Vector3::zeros(), // Zero rates
            &controls,
        );

        // --- Calculate Expected DAMPING contribution ONLY ---
        let q_dyn = air_data.dynamic_pressure;
        let s_area = geometry.wing_area;
        let mac = geometry.mac;
        let v_denom = 2.0 * speed + 1e-9;
        let q_hat = (mac / v_denom) * pitch_rate_q; // Non-dimensional pitch rate
        let alpha = alpha_rad.clamp(-10.0 * PI / 180.0, 40.0 * PI / 180.0); // Use clamped alpha

        // Sum of all Cm terms involving q_hat
        let expected_cm_q_contrib = (coeffs.pitch.c_m_q * q_hat)
            + (coeffs.pitch.c_m_alpha_q * alpha * q_hat)
            + (coeffs.pitch.c_m_alpha2_q * q_hat * alpha.powi(2))
            + (coeffs.pitch.c_m_alpha3_q * q_hat * alpha.powi(3));

        let expected_damping_moment_contrib = q_dyn * s_area * mac * expected_cm_q_contrib;

        // --- Call function WITH damping ---
        let (_forces, moments) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &angular_velocity_body, // Non-zero q
            &controls,
        );

        // --- Assertions ---
        // Check that the difference between moments with and without q matches damping contribution
        assert_relative_eq!(
            moments.y - moments_no_q.y,
            expected_damping_moment_contrib,
            epsilon = TEST_EPSILON
        );

        // Check sign based on primary damping term Cm_q (usually negative)
        if coeffs.pitch.c_m_q < 0.0 && pitch_rate_q > 0.0 {
            assert!(
                expected_damping_moment_contrib < 0.0,
                "Expected negative damping moment contribution for positive q and negative Cm_q"
            );
            assert!(
                moments.y < moments_no_q.y,
                "Pitch moment should decrease due to pitch damping"
            );
        } else if coeffs.pitch.c_m_q > 0.0 && pitch_rate_q > 0.0 {
            assert!(
                expected_damping_moment_contrib > 0.0,
                "Expected positive moment contribution for positive q and positive Cm_q"
            );
            assert!(
                moments.y > moments_no_q.y,
                "Pitch moment should increase due to positive pitch damping"
            );
        }

        // Check other moments haven't changed significantly (expect base Cm0 moment)
        // let expected_my_base =
        //     air_data.dynamic_pressure * geometry.wing_area * geometry.mac * coeffs.pitch.c_m_0; // Re-calc base M
        //                                                                                         // Roll and Yaw moments should be near zero (as p=0, r=0, beta=0, controls=0)
        assert_relative_eq!(moments.x, 0.0, epsilon = q_dyn * 1e-3);
        assert_relative_eq!(moments.z, 0.0, epsilon = q_dyn * 1e-3);
    }

    #[test]
    fn test_roll_damping() {
        // Scenario: Roll rate 'p' present, check damping moment 'L' and induced yaw 'N'
        let speed = 50.0;
        let roll_rate_p: f64 = 0.2; // rad/s (positive roll rate = right wing down)
        let geometry = default_geometry();
        let coeffs = default_coeffs();
        let air_data = create_air_data(speed, 0.0, 0.0, STD_DENSITY); // Neutral alpha/beta
        let angular_velocity_body = Vector3::new(roll_rate_p, 0.0, 0.0); // Only roll rate
        let controls = default_controls();

        // --- Calculate Expected Moment WITHOUT roll rate 'p' ---
        // This gives us the baseline moments (e.g., from Cm0)
        let (_forces_no_p, moments_no_p) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &Vector3::zeros(), // Zero rates
            &controls,
        );

        // --- Calculate Expected DAMPING (Mx) and INDUCED (Mz) contributions from 'p' ONLY ---
        let q_dyn = air_data.dynamic_pressure;
        let s_area = geometry.wing_area;
        let span = geometry.wing_span;
        let mac = geometry.mac; // Needed for pitch moment baseline check
        let v_denom = 2.0 * speed + 1e-9;

        // Clamp p rate as done inside the function (though 0.2 is likely within limits)
        let p = roll_rate_p.clamp(-100.0 * PI / 180.0, 100.0 * PI / 180.0);
        let p_hat = (span / v_denom) * p; // Non-dimensional roll rate

        // Expected Roll Moment contribution from damping (Cl_p * p_hat)
        let expected_cl_p_contrib = coeffs.roll.c_l_p * p_hat;
        let expected_damping_moment_contrib = q_dyn * s_area * span * expected_cl_p_contrib;

        // Expected Yaw Moment contribution from roll rate (Cn_p * p_hat)
        let expected_cn_p_contrib = coeffs.yaw.c_n_p * p_hat;
        let expected_yaw_moment_from_p = q_dyn * s_area * span * expected_cn_p_contrib;

        // Expected Pitch moment baseline (should be same as moments_no_p.y)
        let expected_my_base = moments_no_p.y;
        // Sanity check calculation for baseline pitch moment
        let expected_my_base_calc =
            air_data.dynamic_pressure * geometry.wing_area * mac * coeffs.pitch.c_m_0;
        assert_relative_eq!(
            expected_my_base,
            expected_my_base_calc,
            epsilon = TEST_EPSILON
        );

        // --- Call function WITH roll rate 'p' ---
        let (_forces, moments) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &angular_velocity_body, // Non-zero p
            &controls,
        );

        // --- Assertions ---
        // Check Roll Moment Damping component (difference from baseline)
        assert_relative_eq!(
            moments.x - moments_no_p.x, // Isolate the change due to p
            expected_damping_moment_contrib,
            epsilon = TEST_EPSILON
        );

        // Check Roll Moment Damping Sign (Optional but good)
        if coeffs.roll.c_l_p < 0.0 && roll_rate_p > 0.0 {
            assert!(
                expected_damping_moment_contrib < 0.0,
                "Expected negative damping moment contribution for positive p and negative Cl_p"
            );
            assert!(
                moments.x < moments_no_p.x,
                "Roll moment should decrease due to roll damping"
            );
        } else if coeffs.roll.c_l_p > 0.0 && roll_rate_p > 0.0 {
            assert!(
                expected_damping_moment_contrib > 0.0,
                "Expected positive moment contribution for positive p and positive Cl_p"
            );
            assert!(
                moments.x > moments_no_p.x,
                "Roll moment should increase due to positive roll damping"
            );
        }

        // Check Pitch Moment (should remain at baseline, same as moments_no_p.y)
        assert_relative_eq!(
            moments.y,
            expected_my_base, // Compare against the baseline calculated without p
            epsilon = expected_my_base.abs().max(1.0) * 1e-3  // Allow small tolerance
        );

        // Check Yaw Moment (should equal the contribution from Cn_p, baseline Mz is 0)
        // assert_relative_eq!(moments.z, 0.0, epsilon = q_dyn * 1e-3); // OLD assertion
        assert_relative_eq!(
            moments.z, // Compare the total Mz against the expected contribution (since baseline Mz is 0)
            expected_yaw_moment_from_p,
            epsilon = TEST_EPSILON // Use tight epsilon as we expect this exact term contribution
        );

        // Optional: Check sign of induced yaw moment based on Cn_p
        if coeffs.yaw.c_n_p != 0.0 && roll_rate_p != 0.0 {
            assert!(
                moments.z.signum() == expected_yaw_moment_from_p.signum(),
                "Induced yaw moment sign mismatch"
            );
        }
    }

    #[test]
    fn test_combined_effects_sanity_check() {
        // Scenario: Multiple non-zero inputs, check for finite outputs and expected signs
        let speed = 60.0;
        let alpha_deg = 8.0;
        let beta_deg = -5.0; // Left sideslip
        let p = 0.1; // Positive roll rate
        let q = -0.05; // Negative pitch rate (nose down)
        let r = 0.02; // Positive yaw rate (nose right)
        let elevator = 0.1; // Positive elevator (nose up command / TE up)
        let aileron = -0.1; // Negative aileron (left roll command)
        let rudder = 0.05; // Positive rudder (yaw right command / TE left)

        let geometry = default_geometry();
        let coeffs = default_coeffs();
        let air_data = create_air_data(
            speed,
            alpha_deg * PI / 180.0,
            beta_deg * PI / 180.0,
            STD_DENSITY,
        );
        let angular_velocity_body = Vector3::new(p, q, r);
        let mut controls = default_controls();
        controls.elevator = elevator;
        controls.aileron = aileron;
        controls.rudder = rudder;

        // --- Call the function ---
        let (forces, moments) = calculate_aerodynamic_forces_moments(
            &geometry,
            &coeffs,
            &air_data,
            &angular_velocity_body,
            &controls,
        );

        // --- Assertions ---
        // Basic Checks: Finite and non-zero (usually expected in combined cases)
        assert!(
            forces.iter().all(|v| v.is_finite()),
            "Forces should be finite"
        );
        assert!(
            moments.iter().all(|v| v.is_finite()),
            "Moments should be finite"
        );

        // Check forces (expect non-zero due to alpha/beta/controls)
        assert!(
            forces.x.abs() > TEST_EPSILON,
            "Expected non-zero Drag/Force (Fx)"
        );
        assert!(
            forces.y.abs() > TEST_EPSILON,
            "Expected non-zero Sideforce (Fy) due to Beta/Controls/Rates"
        );
        assert!(
            forces.z.abs() > TEST_EPSILON,
            "Expected non-zero Lift/Force (Fz) due to Alpha/Controls/Rates"
        );

        // Check moments (expect non-zero due to combined effects)
        assert!(
            moments.x.abs() > TEST_EPSILON,
            "Expected non-zero Roll Moment (Mx)"
        );
        assert!(
            moments.y.abs() > TEST_EPSILON,
            "Expected non-zero Pitch Moment (My)"
        );
        assert!(
            moments.z.abs() > TEST_EPSILON,
            "Expected non-zero Yaw Moment (Mz)"
        );

        // Sanity check signs based on dominant inputs (qualitative)
        assert!(
            forces.x < 0.0,
            "Drag component should generally make Fx negative"
        ); // Drag usually dominates Fx
           // Check side force sign based on beta (Corrected)
        if coeffs.side_force.c_y_beta < 0.0 && beta_deg < 0.0 {
            // Check if beta effect dominates other Cy terms (controls, rates)
            let cy_beta_term = coeffs.side_force.c_y_beta * (beta_deg * PI / 180.0);
            // Rough estimate - actual check would need full coefficient calculation
            if cy_beta_term.abs() > 0.1 {
                // Heuristic: if beta term is significant
                assert!(
                    forces.y > 0.0,
                    "Negative Beta should yield positive Fy (assuming negative Cy_beta dominates)"
                );
            } else {
                warn!("WARN: Side force sign check inconclusive due to combined effects.");
            }
        }
        assert!(
            forces.z < 0.0,
            "Positive Alpha yields Lift, which should dominate to make Fz negative"
        ); // Lift usually dominates Fz

        info!("Combined effects forces: {:?}", forces);
        info!("Combined effects moments: {:?}", moments);
    }
}
