use crate::{
    components::{
        AircraftControlSurfaces,
        Force,
        ForceCategory,
        FullAircraftConfig,
        LongitudinalBounds, // Need bounds struct
        Moment,
        PropulsionState, // Added missing import
        ReferenceFrame,
        SpatialComponent,
        TrimCondition,
        TrimSolverConfig, // Added missing import
    },
    resources::PhysicsConfig,
    systems::{
        calculate_aerodynamic_forces_moments, calculate_air_data, calculate_engine_outputs,
        calculate_net_forces_moments, EngineOutputs,
    },
};
use argmin::core::{CostFunction, Error as ArgminError, Gradient};
use nalgebra::{UnitQuaternion, Vector3}; // For logging inside cost/gradient if needed with debug level

#[derive(Clone)] // Needed for argmin Executor
pub struct TrimProblem<'a> {
    pub aircraft_config: &'a FullAircraftConfig,
    pub physics_config: &'a PhysicsConfig,
    pub solver_config: &'a TrimSolverConfig,
    pub target_condition: TrimCondition,
    pub initial_spatial: SpatialComponent, // For position/density, initial velocity magnitude etc.
    pub initial_prop_state: PropulsionState, // For engine configs/count
    pub initial_density: f64,
}

// Helper to get clamped state variables from parameter vector [alpha, elevator, power_lever]
pub fn params_to_state_inputs(param: &[f64], bounds: &LongitudinalBounds) -> (f64, f64, f64) {
    let alpha = param[0].clamp(bounds.alpha_range.0, bounds.alpha_range.1);
    let elevator = param[1].clamp(bounds.elevator_range.0, bounds.elevator_range.1);
    let power_lever = param[2].clamp(bounds.throttle_range.0, bounds.throttle_range.1);
    (alpha, elevator, power_lever)
}

impl CostFunction for TrimProblem<'_> {
    type Param = Vec<f64>;
    type Output = f64; // Cost is sum of squared residuals

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, ArgminError> {
        if param.len() != 3 {
            return Err(ArgminError::msg(
                "Param vector must have length 3 [alpha, elevator, power_lever]",
            ));
        }

        // 1. Decode & Clamp Parameters
        let (alpha, elevator, power_lever) =
            params_to_state_inputs(param, &self.solver_config.longitudinal_bounds);

        // 2. Define Hypothetical State
        let (target_airspeed, target_gamma) = match self.target_condition {
            TrimCondition::StraightAndLevel { airspeed } => (airspeed, 0.0),
            TrimCondition::SteadyClimb { airspeed, gamma } => (airspeed, gamma),
            TrimCondition::CoordinatedTurn { .. } => {
                return Err(ArgminError::msg("Coordinated Turn not implemented yet"))
            } // TODO
        };
        let theta = (alpha + target_gamma).clamp(
            self.solver_config.longitudinal_bounds.theta_range.0,
            self.solver_config.longitudinal_bounds.theta_range.1,
        );
        let attitude = UnitQuaternion::from_euler_angles(0.0, theta, 0.0);
        let velocity = Vector3::new(
            target_airspeed * target_gamma.cos(),
            0.0,
            -target_airspeed * target_gamma.sin(),
        );
        let hypothetical_spatial = SpatialComponent {
            velocity,
            attitude,
            angular_velocity: Vector3::zeros(),
            ..self.initial_spatial
        };
        let hypothetical_controls = AircraftControlSurfaces {
            elevator,
            power_lever,
            aileron: 0.0,
            rudder: 0.0,
        };

        // 3. Calculate Forces/Moments using PURE functions
        let density = self.initial_density;
        let wind = Vector3::zeros(); // Approximation - TODO: Get from environment
        let air_data_values = calculate_air_data(
            &hypothetical_spatial.velocity,
            &hypothetical_spatial.attitude,
            &wind,
            density,
            0.5,
        );
        let (aero_f, aero_m) = calculate_aerodynamic_forces_moments(
            &self.aircraft_config.geometry,
            &self.aircraft_config.aero_coef,
            &air_data_values,
            &hypothetical_spatial.angular_velocity,
            &hypothetical_controls,
        );

        let mut external_forces: Vec<Force> = Vec::new();

        // Add Aero Force
        external_forces.push(Force {
            vector: aero_f,
            category: ForceCategory::Aerodynamic,
            frame: ReferenceFrame::Body,
            point: None, // Aero forces/moments referenced to CG
        });

        // Calculate and Add Individual Engine Forces
        for (engine_config, current_engine_state) in self
            .aircraft_config
            .propulsion
            .engines
            .iter()
            .zip(self.initial_prop_state.engine_states.iter())
        {
            let mut temp_engine_state = current_engine_state.clone();
            temp_engine_state.power_lever = hypothetical_controls.power_lever;
            temp_engine_state.thrust_fraction = temp_engine_state.power_lever; // Steady state approx
            temp_engine_state.running = temp_engine_state.thrust_fraction > 0.01;

            // Assume calculate_engine_outputs returns EngineOutputs struct
            // which contains a Force struct with the correct application point.
            let outputs: EngineOutputs = calculate_engine_outputs(
                engine_config,
                &temp_engine_state,
                air_data_values.density,
                air_data_values.true_airspeed,
            );

            // Add the full force component (vector + point) for this engine
            // Only add if the force is significant to avoid clutter? Optional.
            if outputs.force_component.vector.norm_squared() > 1e-9 {
                external_forces.push(outputs.force_component);
            }
            // NOTE: We are not storing fuel flow or separate prop moment here,
            // as the moment is calculated from the force component's point later.
        }

        // Define External Moments (currently only aero)
        let external_moments = [Moment {
            vector: aero_m,
            category: ForceCategory::Aerodynamic,
            frame: ReferenceFrame::Body,
        }];

        if self.solver_config.debug_level > 0 {
            // Use debug level 1 for basic checks
            println!("\n--- Trim Cost Calculation ---");
            println!(
                "  Input Param (alpha, elev, pwr): [{:.4}, {:.4}, {:.4}]",
                param[0], param[1], param[2]
            );
            println!(
                "  Clamped Input (alpha, elev, pwr): [{:.4}, {:.4}, {:.4}]",
                alpha, elevator, power_lever
            );
            println!("  Hypothetical State:");
            println!("    Target Airspeed: {:.2}", target_airspeed);
            println!("    Target Gamma: {:.2}", target_gamma.to_degrees());
            println!("    Theta (alpha+gamma): {:.2}", theta.to_degrees());
            println!(
                "    Velocity (World): [{:.2}, {:.2}, {:.2}]",
                velocity.x, velocity.y, velocity.z
            );
            println!("    Attitude (Quat): {:?}", attitude);
            println!("  Air Data Values:");
            println!("    TAS: {:.2}", air_data_values.true_airspeed);
            println!("    Alpha: {:.2} deg", air_data_values.alpha.to_degrees());
            println!("    Beta: {:.2} deg", air_data_values.beta.to_degrees());
            println!(
                "    Dyn Pressure (q): {:.2}",
                air_data_values.dynamic_pressure
            );
            println!("  Calculated Forces/Moments (Body Frame):");
            println!(
                "    Aero Force: [{:.2}, {:.2}, {:.2}]",
                aero_f.x, aero_f.y, aero_f.z
            );
            println!(
                "    Aero Moment: [{:.2}, {:.2}, {:.2}]",
                aero_m.x, aero_m.y, aero_m.z
            );
        }

        let (net_force_inertial, net_moment_body, grav_body_vec) = calculate_net_forces_moments(
            &external_forces,
            &external_moments,
            &hypothetical_spatial.attitude,
            self.aircraft_config.mass.mass,
            &self.physics_config.gravity,
        );

        if self.solver_config.debug_level > 0 {
            println!("  Net Forces/Moments:");
            println!(
                "    Net Force (Inertial): [{:.2}, {:.2}, {:.2}]",
                net_force_inertial.x, net_force_inertial.y, net_force_inertial.z
            );
            println!(
                "    Net Moment (Body): [{:.2}, {:.2}, {:.2}]",
                net_moment_body.x, net_moment_body.y, net_moment_body.z
            );
            println!(
                "    Gravity Vector (Body): [{:.2}, {:.2}, {:.2}]",
                grav_body_vec.x, grav_body_vec.y, grav_body_vec.z
            );
        }

        // 4. Calculate Residuals & Cost
        let pitch_moment_residual = net_moment_body.y / 10000.0;
        let vertical_force_residual = net_force_inertial.z / 10000.0;
        let horizontal_force_residual = net_force_inertial.x / 5000.0;
        let cost = pitch_moment_residual.powi(2)
            + vertical_force_residual.powi(2)
            + horizontal_force_residual.powi(2);

        // Optional Debug Print
        if self.solver_config.debug_level > 0 {
            println!("  Residuals:");
            println!("    Pitch Moment: {:.4e}", pitch_moment_residual);
            println!("    Vertical Force: {:.4e}", vertical_force_residual);
            println!("    Horizontal Force: {:.4e}", horizontal_force_residual);
            println!("  Calculated Cost: {:.6e}", cost);
            println!("--- End Trim Cost Calculation ---\n");
        }

        Ok(cost)
    }
}

// Implement Gradient using Finite Differences
impl Gradient for TrimProblem<'_> {
    type Param = Vec<f64>;
    type Gradient = Vec<f64>;

    fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, ArgminError> {
        let epsilon = 1e-7; // Step size for finite difference
        let mut grad = vec![0.0; param.len()];
        // let cost_center = self.cost(param)?; // Calculate cost at the center point once

        for i in 0..param.len() {
            let mut param_plus = param.clone();
            // Only perturb one parameter
            param_plus[i] += epsilon;
            // Clamping is handled inside cost function now
            let cost_plus = self.cost(&param_plus)?;
            // Forward difference approximation
            // grad[i] = (cost_plus - cost_center) / epsilon;

            // Or use Central difference (more accurate but more costly):
            let mut param_minus = param.clone();
            param_minus[i] -= epsilon;
            let cost_minus = self.cost(&param_minus)?;
            grad[i] = (cost_plus - cost_minus) / (2.0 * epsilon);
        }
        Ok(grad)
    }
}
