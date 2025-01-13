use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    components::{
        AircraftControlSurfaces, FullAircraftConfig, FullAircraftState, TrimCondition,
        TrimResiduals, TrimResult, TrimSolverConfig, TrimState,
    },
    resources::PhysicsConfig,
    systems::VirtualPhysics,
};

/// Modified trim solver using virtual physics
pub struct TrimSolver {
    pub virtual_physics: VirtualPhysics,
    pub config: FullAircraftConfig,
    pub condition: TrimCondition,
    pub settings: TrimSolverConfig,
    pub current_state: TrimState,
    pub best_state: TrimState,
    pub best_cost: f64,
    pub iteration: usize,
    pub virtual_aircraft: Entity,
}

impl TrimSolver {
    pub fn new(
        settings: TrimSolverConfig,
        config: FullAircraftConfig,
        condition: TrimCondition,
        physics_config: PhysicsConfig,
    ) -> Self {
        let mut virtual_physics = VirtualPhysics::new(physics_config, 1.0 / 120.0);

        // Create initial state
        let initial_state = FullAircraftState::default();
        let virtual_aircraft = virtual_physics.spawn_aircraft(&initial_state, &config);

        Self {
            virtual_physics,
            config,
            condition,
            settings,
            current_state: TrimState::default(),
            best_state: TrimState::default(),
            best_cost: f64::MAX,
            iteration: 0,
            virtual_aircraft,
        }
    }

    pub fn initialize(&mut self, initial_guess: TrimState) {
        self.current_state = initial_guess;
        self.best_state = initial_guess;
        self.best_cost = f64::MAX;
        self.iteration = 0;
    }

    pub fn iterate(&mut self) {
        // 1. Calculate residuals for current state
        let residuals = self.calculate_residuals();
        let cost = self.calculate_cost(&residuals);

        // 2. Update best solution if current is better
        if cost < self.best_cost {
            self.best_cost = cost;
            self.best_state = self.current_state;
        }

        // 3. Update state using chosen optimization method
        if self.settings.use_gradient_refinement {
            self.gradient_optimization_step();
        } else {
            self.pattern_search_step();
        }

        self.iteration += 1;
    }

    pub fn has_converged(&self) -> bool {
        self.best_cost < self.settings.cost_tolerance
    }

    pub fn get_best_solution(&mut self) -> TrimResult {
        TrimResult {
            state: self.best_state,
            converged: self.has_converged(),
            cost: self.best_cost,
            iterations: self.iteration,
            residuals: self.calculate_residuals(),
        }
    }

    fn calculate_residuals(&mut self) -> TrimResiduals {
        // Create state from trim variables
        let mut state = FullAircraftState::default();

        // Set control surfaces
        state.control_surfaces = AircraftControlSurfaces {
            elevator: self.current_state.elevator,
            aileron: self.current_state.aileron,
            rudder: self.current_state.rudder,
            power_lever: self.current_state.power_lever,
        };

        // Set attitude
        state.spatial.attitude = UnitQuaternion::from_euler_angles(
            self.current_state.phi,
            self.current_state.theta,
            0.0, // yaw not considered in trim
        );

        // Set velocities based on trim condition
        let (vx, vy, vz) = match self.condition {
            TrimCondition::StraightAndLevel { airspeed } => {
                let alpha = self.current_state.alpha;
                (airspeed * alpha.cos(), 0.0, -airspeed * alpha.sin())
            }
            TrimCondition::SteadyClimb { airspeed, gamma } => {
                let alpha = self.current_state.alpha;
                (
                    airspeed * (alpha - gamma).cos(),
                    0.0,
                    -airspeed * (alpha - gamma).sin(),
                )
            }
            TrimCondition::CoordinatedTurn {
                airspeed,
                bank_angle: _bank_angle, // TODO: Use bank angle
            } => {
                let alpha = self.current_state.alpha;
                let beta = self.current_state.beta;
                (
                    airspeed * alpha.cos() * beta.cos(),
                    airspeed * beta.sin(),
                    -airspeed * alpha.sin() * beta.cos(),
                )
            }
        };
        state.spatial.velocity = Vector3::new(vx, vy, vz);

        // Update virtual aircraft state
        self.virtual_physics
            .set_state(self.virtual_aircraft, &state);

        // Run virtual physics for a few steps to stabilize
        self.virtual_physics.run_steps(self.virtual_aircraft, 10);

        // Calculate forces at final state
        let (forces, moments) = self.virtual_physics.calculate_forces(self.virtual_aircraft);
        let final_state = self.virtual_physics.get_state(self.virtual_aircraft);

        TrimResiduals {
            forces,
            moments,
            gamma_error: match self.condition {
                TrimCondition::SteadyClimb { gamma, .. } => {
                    let actual_gamma =
                        (-final_state.spatial.velocity.z / final_state.spatial.velocity.x).atan();
                    gamma - actual_gamma
                }
                _ => 0.0,
            },
            mu_error: match self.condition {
                TrimCondition::CoordinatedTurn { bank_angle, .. } => {
                    bank_angle - self.current_state.phi
                }
                _ => 0.0,
            },
        }
    }

    fn calculate_cost(&self, residuals: &TrimResiduals) -> f64 {
        // Weight different residual components
        let force_weight = 1.0;
        let moment_weight = 10.0;
        let gamma_weight = 5.0;
        let mu_weight = 5.0;

        // Calculate individual costs
        let force_cost = residuals.forces.norm_squared();
        let moment_cost = residuals.moments.norm_squared();
        let gamma_cost = residuals.gamma_error.powi(2);
        let mu_cost = residuals.mu_error.powi(2);

        // Combine weighted costs
        force_weight * force_cost
            + moment_weight * moment_cost
            + gamma_weight * gamma_cost
            + mu_weight * mu_cost
    }

    fn gradient_optimization_step(&mut self) {
        // Calculate gradients using finite differences
        let gradients = self.calculate_gradients();

        // Simple gradient descent with line search
        let learning_rates = [0.1, 0.01, 0.001];

        let initial_residuals = self.calculate_residuals();
        let mut best_cost = self.calculate_cost(&initial_residuals);
        let mut best_state = self.current_state;

        for &rate in &learning_rates {
            let mut test_state = self.current_state;

            // Update all state variables using gradients
            test_state.elevator -= rate * gradients[0];
            test_state.aileron -= rate * gradients[1];
            test_state.rudder -= rate * gradients[2];
            test_state.power_lever -= rate * gradients[3];
            test_state.alpha -= rate * gradients[4];
            test_state.beta -= rate * gradients[5];
            test_state.phi -= rate * gradients[6];
            test_state.theta -= rate * gradients[7];

            // Enforce bounds
            self.enforce_bounds(&mut test_state);

            // Test new state
            self.current_state = test_state;

            let test_residuals = self.calculate_residuals();
            let test_cost = self.calculate_cost(&test_residuals);

            if test_cost < best_cost {
                best_cost = test_cost;
                best_state = test_state;
            }
        }

        self.current_state = best_state;
    }

    fn pattern_search_step(&mut self) {
        let deltas = [0.1, 0.01, 0.001];

        let initial_residuals = self.calculate_residuals();
        let mut best_cost = self.calculate_cost(&initial_residuals);
        let mut best_variation = None;

        // Try variations of each variable
        for &delta in &deltas {
            for var in 0..8 {
                for &sign in &[-1.0, 1.0] {
                    let mut test_state = self.current_state;

                    // Modify one variable at a time
                    match var {
                        0 => test_state.elevator += sign * delta,
                        1 => test_state.aileron += sign * delta,
                        2 => test_state.rudder += sign * delta,
                        3 => test_state.power_lever += sign * delta,
                        4 => test_state.alpha += sign * delta,
                        5 => test_state.beta += sign * delta,
                        6 => test_state.phi += sign * delta,
                        7 => test_state.theta += sign * delta,
                        _ => unreachable!(),
                    }

                    self.enforce_bounds(&mut test_state);
                    self.current_state = test_state;
                    let test_residuals = self.calculate_residuals();
                    let test_cost = self.calculate_cost(&test_residuals);

                    if test_cost < best_cost {
                        best_cost = test_cost;
                        best_variation = Some((var, sign * delta));
                    }
                }
            }
        }

        // Apply best variation found
        if let Some((var, delta)) = best_variation {
            let mut new_state = self.current_state;
            match var {
                0 => new_state.elevator += delta,
                1 => new_state.aileron += delta,
                2 => new_state.rudder += delta,
                3 => new_state.power_lever += delta,
                4 => new_state.alpha += delta,
                5 => new_state.beta += delta,
                6 => new_state.phi += delta,
                7 => new_state.theta += delta,
                _ => unreachable!(),
            }
            self.enforce_bounds(&mut new_state);
            self.current_state = new_state;
        }
    }

    // TODO: Abstract this somehow
    fn calculate_gradients(&mut self) -> Vec<f64> {
        let epsilon = 1e-6;
        let mut gradients = Vec::with_capacity(8);

        // Calculate gradient for elevator
        {
            let original = self.current_state.elevator;
            let (min, max) = self.settings.bounds.elevator_range;

            self.current_state.elevator = (original + epsilon).clamp(min, max);
            let forward_residuals = self.calculate_residuals();
            let cost_plus = self.calculate_cost(&forward_residuals);

            self.current_state.elevator = (original - epsilon).clamp(min, max);
            let backward_residuals = self.calculate_residuals();
            let cost_minus = self.calculate_cost(&backward_residuals);

            gradients.push((cost_plus - cost_minus) / (2.0 * epsilon));
            self.current_state.elevator = original;
        }

        // Calculate gradient for aileron
        {
            let original = self.current_state.aileron;
            let (min, max) = self.settings.bounds.aileron_range;

            self.current_state.aileron = (original + epsilon).clamp(min, max);
            let forward_residuals = self.calculate_residuals();
            let cost_plus = self.calculate_cost(&forward_residuals);

            self.current_state.aileron = (original - epsilon).clamp(min, max);
            let backward_residuals = self.calculate_residuals();
            let cost_minus = self.calculate_cost(&backward_residuals);

            gradients.push((cost_plus - cost_minus) / (2.0 * epsilon));
            self.current_state.aileron = original;
        }

        // Calculate gradient for rudder
        {
            let original = self.current_state.rudder;
            let (min, max) = self.settings.bounds.rudder_range;

            self.current_state.rudder = (original + epsilon).clamp(min, max);
            let forward_residuals = self.calculate_residuals();
            let cost_plus = self.calculate_cost(&forward_residuals);

            self.current_state.rudder = (original - epsilon).clamp(min, max);
            let backward_residuals = self.calculate_residuals();
            let cost_minus = self.calculate_cost(&backward_residuals);

            gradients.push((cost_plus - cost_minus) / (2.0 * epsilon));
            self.current_state.rudder = original;
        }

        // Calculate gradient for power_lever
        {
            let original = self.current_state.power_lever;
            let (min, max) = self.settings.bounds.throttle_range;

            self.current_state.power_lever = (original + epsilon).clamp(min, max);
            let forward_residuals = self.calculate_residuals();
            let cost_plus = self.calculate_cost(&forward_residuals);

            self.current_state.power_lever = (original - epsilon).clamp(min, max);
            let backward_residuals = self.calculate_residuals();
            let cost_minus = self.calculate_cost(&backward_residuals);

            gradients.push((cost_plus - cost_minus) / (2.0 * epsilon));
            self.current_state.power_lever = original;
        }

        // Calculate gradient for alpha
        {
            let original = self.current_state.alpha;
            let (min, max) = self.settings.bounds.alpha_range;

            self.current_state.alpha = (original + epsilon).clamp(min, max);
            let forward_residuals = self.calculate_residuals();
            let cost_plus = self.calculate_cost(&forward_residuals);

            self.current_state.alpha = (original - epsilon).clamp(min, max);
            let backward_residuals = self.calculate_residuals();
            let cost_minus = self.calculate_cost(&backward_residuals);

            gradients.push((cost_plus - cost_minus) / (2.0 * epsilon));
            self.current_state.alpha = original;
        }

        // Calculate gradient for beta
        {
            let original = self.current_state.beta;
            let (min, max) = self.settings.bounds.beta_range;

            self.current_state.beta = (original + epsilon).clamp(min, max);
            let forward_residuals = self.calculate_residuals();
            let cost_plus = self.calculate_cost(&forward_residuals);

            self.current_state.beta = (original - epsilon).clamp(min, max);
            let backward_residuals = self.calculate_residuals();
            let cost_minus = self.calculate_cost(&backward_residuals);

            gradients.push((cost_plus - cost_minus) / (2.0 * epsilon));
            self.current_state.beta = original;
        }

        // Calculate gradient for phi
        {
            let original = self.current_state.phi;
            let (min, max) = self.settings.bounds.phi_range;

            self.current_state.phi = (original + epsilon).clamp(min, max);
            let forward_residuals = self.calculate_residuals();
            let cost_plus = self.calculate_cost(&forward_residuals);

            self.current_state.phi = (original - epsilon).clamp(min, max);
            let backward_residuals = self.calculate_residuals();
            let cost_minus = self.calculate_cost(&backward_residuals);

            gradients.push((cost_plus - cost_minus) / (2.0 * epsilon));
            self.current_state.phi = original;
        }

        // Calculate gradient for theta
        {
            let original = self.current_state.theta;
            let (min, max) = self.settings.bounds.theta_range;

            self.current_state.theta = (original + epsilon).clamp(min, max);
            let forward_residuals = self.calculate_residuals();
            let cost_plus = self.calculate_cost(&forward_residuals);

            self.current_state.theta = (original - epsilon).clamp(min, max);
            let backward_residuals = self.calculate_residuals();
            let cost_minus = self.calculate_cost(&backward_residuals);

            gradients.push((cost_plus - cost_minus) / (2.0 * epsilon));
            self.current_state.theta = original;
        }

        gradients
    }

    fn enforce_bounds(&self, state: &mut TrimState) {
        let bounds = &self.settings.bounds;

        state.elevator = state
            .elevator
            .clamp(bounds.elevator_range.0, bounds.elevator_range.1);
        state.aileron = state
            .aileron
            .clamp(bounds.aileron_range.0, bounds.aileron_range.1);
        state.rudder = state
            .rudder
            .clamp(bounds.rudder_range.0, bounds.rudder_range.1);
        state.power_lever = state
            .power_lever
            .clamp(bounds.throttle_range.0, bounds.throttle_range.1);
        state.alpha = state
            .alpha
            .clamp(bounds.alpha_range.0, bounds.alpha_range.1);
        state.beta = state.beta.clamp(bounds.beta_range.0, bounds.beta_range.1);
        state.phi = state.phi.clamp(bounds.phi_range.0, bounds.phi_range.1);
        state.theta = state
            .theta
            .clamp(bounds.theta_range.0, bounds.theta_range.1);
    }
}
