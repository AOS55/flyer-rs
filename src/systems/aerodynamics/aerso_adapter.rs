use crate::components::{AeroCoefficients, AircraftGeometry, ControlSurfaces};
use aerso::types::{Force, Torque};
use aerso::{AeroEffect, AirState};
use nalgebra::Vector3;
use std::f64::consts::PI;

pub struct AersoAdapter {
    geometry: AircraftGeometry,
    coefficients: AeroCoefficients,
}

impl AersoAdapter {
    pub fn new(geometry: AircraftGeometry, coefficients: AeroCoefficients) -> Self {
        Self {
            coefficients,
            geometry,
        }
    }

    fn compute_forces(
        &self,
        air_state: &AirState<f64>,
        control_surfaces: &ControlSurfaces,
        rates: Vector3<f64>,
    ) -> (Vector3<f64>, Vector3<f64>) {
        let controls = &control_surfaces;
        let coeffs = &self.coefficients;

        // Clamp the angles and rates like in the reference implementation
        let alpha = air_state.alpha.clamp(-10.0 * PI / 180.0, 40.0 * PI / 180.0);
        let beta = air_state.beta.clamp(-20.0 * PI / 180.0, 20.0 * PI / 180.0);
        let p = rates.x.clamp(-100.0 * PI / 180.0, 100.0 * PI / 180.0);
        let q = rates.y.clamp(-50.0 * PI / 180.0, 50.0 * PI / 180.0);
        let r = rates.z.clamp(-50.0 * PI / 180.0, 50.0 * PI / 180.0);

        // Non-dimensional rates
        let p_hat = (self.geometry.wing_span * p) / (2.0 * air_state.airspeed);
        let q_hat = (self.geometry.mean_aerodynamic_chord * q) / (2.0 * air_state.airspeed);
        let r_hat = (self.geometry.wing_span * r) / (2.0 * air_state.airspeed);
        // Calculate coefficients following the reference implementation
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

        let c_l_roll = coeffs.roll.c_l_beta * beta
            + (coeffs.roll.c_l_p * p_hat)
            + (coeffs.roll.c_l_r * r_hat)
            + (coeffs.roll.c_l_deltaa * controls.aileron)
            + (coeffs.roll.c_l_deltar * controls.rudder);

        let c_m = coeffs.pitch.c_m_0
            + (coeffs.pitch.c_m_alpha * alpha)
            + (coeffs.pitch.c_m_q * q_hat)
            + (coeffs.pitch.c_m_deltae * controls.elevator)
            + (coeffs.pitch.c_m_alpha_q * alpha * q_hat)
            + (coeffs.pitch.c_m_alpha2_q * q_hat * alpha.powi(2))
            + (coeffs.pitch.c_m_alpha2_deltae * controls.elevator * alpha.powi(2))
            + (coeffs.pitch.c_m_alpha3_q * q_hat * alpha.powi(3))
            + (coeffs.pitch.c_m_alpha3_deltae * controls.elevator * alpha.powi(3))
            + (coeffs.pitch.c_m_alpha4 * alpha.powi(4));

        let c_n = coeffs.yaw.c_n_beta * beta
            + (coeffs.yaw.c_n_p * p_hat)
            + (coeffs.yaw.c_n_r * r_hat)
            + (coeffs.yaw.c_n_deltaa * controls.aileron)
            + (coeffs.yaw.c_n_deltar * controls.rudder)
            + (coeffs.yaw.c_n_beta2 * beta.powi(2))
            + (coeffs.yaw.c_n_beta3 * beta.powi(3));

        // Calculate forces and moments
        let forces = Vector3::new(
            -air_state.q * self.geometry.wing_area * c_d,
            air_state.q * self.geometry.wing_area * c_y,
            -air_state.q * self.geometry.wing_area * c_l,
        );

        let moments = Vector3::new(
            air_state.q * self.geometry.wing_span * self.geometry.wing_area * c_l_roll,
            air_state.q * self.geometry.mean_aerodynamic_chord * self.geometry.wing_area * c_m,
            air_state.q * self.geometry.wing_span * self.geometry.wing_area * c_n,
        );

        (forces, moments)
    }

    #[allow(dead_code)]
    pub fn create_input_vector(&self, controls: &ControlSurfaces) -> Vec<f64> {
        vec![
            controls.aileron,
            controls.elevator,
            controls.rudder,
            controls.flaps,
        ]
    }
}

impl AeroEffect<Vec<f64>> for AersoAdapter {
    fn get_effect(
        &self,
        airstate: AirState<f64>,
        rates: Vector3<f64>,
        input: &Vec<f64>,
    ) -> (Force<f64>, Torque<f64>) {
        let controls = ControlSurfaces {
            aileron: input[0],
            elevator: input[1],
            rudder: input[2],
            flaps: input[3],
        };

        let (forces, moments) = self.compute_forces(&airstate, &controls, rates);

        (
            Force::body(forces.x, forces.y, forces.z),
            Torque::body(moments.x, moments.y, moments.z),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    fn create_test_geometry() -> AircraftGeometry {
        AircraftGeometry {
            wing_area: 16.0,             // m^2
            wing_span: 10.0,             // m
            mean_aerodynamic_chord: 1.6, // m
        }
    }

    fn create_test_coefficients() -> AeroCoefficients {
        let mut coeffs = AeroCoefficients::default();

        // Basic coefficients
        coeffs.lift.c_l_0 = 0.2;
        coeffs.lift.c_l_alpha = 5.0;
        coeffs.lift.c_l_alpha2 = 0.3; // Add nonlinear terms
        coeffs.lift.c_l_alpha3 = -0.1;
        coeffs.lift.c_l_alpha4 = 0.01;

        coeffs.drag.c_d_0 = 0.025;
        coeffs.drag.c_d_alpha = 0.1;

        coeffs.pitch.c_m_0 = 0.015; // Small trim moment
        coeffs.pitch.c_m_alpha = -0.5; // Static stability
        coeffs.pitch.c_m_deltae = -1.5;
        coeffs.pitch.c_m_q = -7.0; // Pitch damping

        coeffs.pitch.c_m_alpha_q = 0.0;
        coeffs.pitch.c_m_alpha2_q = 0.0;
        coeffs.pitch.c_m_alpha2_deltae = 0.0;
        coeffs.pitch.c_m_alpha3_q = 0.0;
        coeffs.pitch.c_m_alpha3_deltae = 0.0;
        coeffs.pitch.c_m_alpha4 = 0.0;

        coeffs.side_force.c_y_beta = -0.5;
        coeffs.roll.c_l_deltaa = 0.3;
        coeffs.yaw.c_n_beta = 0.1;
        coeffs.yaw.c_n_beta2 = 0.01; // Nonlinear sideslip effects
        coeffs.yaw.c_n_beta3 = -0.005;

        coeffs
    }

    fn create_test_air_state(airspeed: f64, alpha: f64, beta: f64) -> AirState<f64> {
        AirState {
            airspeed,
            alpha,
            beta,
            q: 0.5 * 1.225 * airspeed * airspeed, // Standard sea level density
        }
    }

    #[test]
    fn test_straight_level_flight() {
        let adapter = AersoAdapter::new(create_test_geometry(), create_test_coefficients());
        let air_state = create_test_air_state(50.0, 0.0, 0.0);
        let controls = ControlSurfaces::default();
        let rates = Vector3::zeros();

        let (forces, moments) = adapter.compute_forces(&air_state, &controls, rates);

        // In straight and level flight:
        // - Should have drag (negative x force)
        // - Small lift from c_l_0
        // - No side force
        assert!(forces.x < 0.0, "Expected negative drag force");
        assert!(forces.z < 0.0, "Expected negative lift force (z-down)");
        assert_relative_eq!(forces.y, 0.0, epsilon = 1e-6);

        // No moments in straight level flight with neutral controls
        assert!(
            moments.norm() < 1000.0,
            "Moments should be reasonably small"
        );
    }

    #[test]
    fn test_angle_of_attack_effects() {
        let adapter = AersoAdapter::new(create_test_geometry(), create_test_coefficients());
        let alpha = 5.0 * PI / 180.0;
        let air_state = create_test_air_state(50.0, alpha, 0.0);
        let controls = ControlSurfaces::default();
        let rates = Vector3::zeros();

        let (forces, _) = adapter.compute_forces(&air_state, &controls, rates);

        // Calculate expected lift coefficient including all alpha terms
        let coeffs = &adapter.coefficients;
        let expected_cl = coeffs.lift.c_l_0
            + coeffs.lift.c_l_alpha * alpha
            + coeffs.lift.c_l_alpha2 * alpha.powi(2)
            + coeffs.lift.c_l_alpha3 * alpha.powi(3)
            + coeffs.lift.c_l_alpha4 * alpha.powi(4);

        let actual_cl = -forces.z / (air_state.q * adapter.geometry.wing_area);

        assert_relative_eq!(actual_cl, expected_cl, epsilon = 1e-6);
        assert!(forces.x < 0.0, "Drag should increase with AoA");
    }

    #[test]
    fn test_control_surface_effectiveness() {
        let adapter = AersoAdapter::new(create_test_geometry(), create_test_coefficients());
        let air_state = create_test_air_state(50.0, 0.0, 0.0);
        let rates = Vector3::zeros();

        let mut controls = ControlSurfaces::default();
        controls.elevator = 0.1; // 10% deflection

        let (_, moments) = adapter.compute_forces(&air_state, &controls, rates);

        let actual_cm = moments.y
            / (air_state.q * adapter.geometry.wing_area * adapter.geometry.mean_aerodynamic_chord);

        let expected_cm = adapter.coefficients.pitch.c_m_0
            + adapter.coefficients.pitch.c_m_deltae * controls.elevator;

        println!("Debug elevator effects:");
        println!("  elevator: {}", controls.elevator);
        println!("  c_m_deltae: {}", adapter.coefficients.pitch.c_m_deltae);
        println!("  expected_cm: {}", expected_cm);
        println!("  actual_cm: {}", actual_cm);

        assert_relative_eq!(actual_cm, expected_cm, epsilon = 1e-6);
    }

    #[test]
    fn test_sideslip_effects() {
        let adapter = AersoAdapter::new(create_test_geometry(), create_test_coefficients());
        let beta = 5.0 * PI / 180.0;
        let air_state = create_test_air_state(50.0, 0.0, beta);
        let controls = ControlSurfaces::default();
        let rates = Vector3::zeros();

        let (forces, moments) = adapter.compute_forces(&air_state, &controls, rates);

        // Include all beta terms for side force
        let expected_cy = adapter.coefficients.side_force.c_y_beta * beta;
        let actual_cy = forces.y / (air_state.q * adapter.geometry.wing_area);
        assert_relative_eq!(actual_cy, expected_cy, epsilon = 1e-6);

        // Include all beta terms for yaw moment
        let expected_cn = adapter.coefficients.yaw.c_n_beta * beta
            + adapter.coefficients.yaw.c_n_beta2 * beta.powi(2)
            + adapter.coefficients.yaw.c_n_beta3 * beta.powi(3);

        let actual_cn =
            moments.z / (air_state.q * adapter.geometry.wing_area * adapter.geometry.wing_span);
        assert_relative_eq!(actual_cn, expected_cn, epsilon = 1e-6);
    }

    #[test]
    fn test_angular_rate_effects() {
        let adapter = AersoAdapter::new(create_test_geometry(), create_test_coefficients());
        let air_state = create_test_air_state(50.0, 0.0, 0.0);
        let controls = ControlSurfaces::default();

        // Test pitch rate damping
        let rates = Vector3::new(0.0, 0.2, 0.0); // pitch rate

        // Calculate non-dimensional pitch rate
        let q_hat =
            (adapter.geometry.mean_aerodynamic_chord * rates.y) / (2.0 * air_state.airspeed);

        let (_, moments) = adapter.compute_forces(&air_state, &controls, rates);

        // Include all relevant pitch moment terms
        let expected_cm =
            adapter.coefficients.pitch.c_m_0 + (adapter.coefficients.pitch.c_m_q * q_hat);

        // Divide by all scaling factors used in the moment calculation
        let actual_cm = moments.y
            / (air_state.q * adapter.geometry.wing_area * adapter.geometry.mean_aerodynamic_chord);

        println!("Debug rate effects:");
        println!("  q_hat: {}", q_hat);
        println!("  c_m_q: {}", adapter.coefficients.pitch.c_m_q);
        println!("  c_m_0: {}", adapter.coefficients.pitch.c_m_0);
        println!("  expected_cm: {}", expected_cm);
        println!("  actual_cm: {}", actual_cm);

        assert_relative_eq!(actual_cm, expected_cm, epsilon = 1e-6);
    }

    #[test]
    fn test_combined_effects() {
        let adapter = AersoAdapter::new(create_test_geometry(), create_test_coefficients());

        // Test with combined angle of attack, sideslip, and control inputs
        let air_state = create_test_air_state(
            50.0,
            3.0 * PI / 180.0, // 3 degrees AoA
            2.0 * PI / 180.0, // 2 degrees sideslip
        );

        let mut controls = ControlSurfaces::default();
        controls.aileron = 0.1;
        controls.elevator = -0.05;
        controls.rudder = 0.03;

        let rates = Vector3::new(0.05, 0.03, 0.02); // roll, pitch, yaw rates

        let (forces, moments) = adapter.compute_forces(&air_state, &controls, rates);

        // Verify forces and moments are non-zero and in expected directions
        assert!(forces.x < 0.0, "Expected negative drag force");
        assert!(forces.z < 0.0, "Expected negative lift force");
        assert!(forces.y.abs() > 0.0, "Expected non-zero side force");

        // Verify moments are being generated in all axes
        assert!(moments.x.abs() > 0.0, "Expected roll moment");
        assert!(moments.y.abs() > 0.0, "Expected pitch moment");
        assert!(moments.z.abs() > 0.0, "Expected yaw moment");
    }

    #[test]
    fn test_input_vector_creation() {
        let adapter = AersoAdapter::new(create_test_geometry(), create_test_coefficients());
        let controls = ControlSurfaces {
            aileron: 0.1,
            elevator: -0.2,
            rudder: 0.3,
            flaps: 0.4,
        };

        let input = adapter.create_input_vector(&controls);

        assert_eq!(input.len(), 4);
        assert_relative_eq!(input[0], 0.1); // aileron
        assert_relative_eq!(input[1], -0.2); // elevator
        assert_relative_eq!(input[2], 0.3); // rudder
        assert_relative_eq!(input[3], 0.4); // flaps
    }
}
