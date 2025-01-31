use argmin::{
    core::{CostFunction, Error, Executor, Gradient, IterState, OptimizationResult},
    solver::{linesearch::MoreThuenteLineSearch, neldermead::NelderMead, quasinewton::LBFGS},
};
use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    components::{
        AircraftControlSurfaces, FullAircraftConfig, PropulsionState, SpatialComponent,
        TrimCondition, TrimResiduals, TrimResult, TrimSolverConfig, TrimState,
    },
    resources::PhysicsConfig,
    systems::VirtualPhysics,
};

#[derive(Debug)]
pub struct TrimOptimizer {
    physics_config: PhysicsConfig,
    initial_spatial: SpatialComponent,
    initial_prop: PropulsionState,
    aircraft_config: FullAircraftConfig,
    condition: TrimCondition,
    settings: TrimSolverConfig,
    characteristic_force: f64,
    characteristic_moment: f64,
}

impl CostFunction for TrimOptimizer {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        let state = TrimState {
            elevator: param[0],
            aileron: param[1],
            rudder: param[2],
            power_lever: param[3],
            alpha: param[4],
            beta: param[5],
            phi: param[6],
            theta: param[7],
        };

        if !self.check_bounds(&state) {
            return Ok(f64::INFINITY);
        }

        let residuals = self.calculate_residuals(&state);

        let force_weight = 1.0;
        let moment_weight = 1.0;
        let gamma_weight = 10.0;
        let mu_weight = 10.0;

        let cost = force_weight * residuals.forces.norm_squared()
            + moment_weight * residuals.moments.norm_squared()
            + gamma_weight * residuals.gamma_error.powi(2)
            + mu_weight * residuals.mu_error.powi(2);

        println!("Cost: {}", cost);

        Ok(cost)
    }
}

impl Gradient for TrimOptimizer {
    type Param = Vec<f64>;
    type Gradient = Vec<f64>;

    fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
        // Implement finite difference gradient calculation
        let eps = 1e-6;
        let n = param.len();
        let mut grad = vec![0.0; n];

        for i in 0..n {
            // Central differences
            let mut param_plus = param.clone();
            let mut param_minus = param.clone();
            param_plus[i] += eps;
            param_minus[i] -= eps;

            let f_plus = self.cost(&param_plus)?;
            let f_minus = self.cost(&param_minus)?;

            if f_plus.is_finite() && f_minus.is_finite() {
                grad[i] = (f_plus - f_minus) / eps;
                // Scale large gradients
                if grad[i].abs() > 1.0 {
                    grad[i] = grad[i].signum() * grad[i].abs().sqrt();
                }
            }
        }

        Ok(grad)
    }
}

impl TrimOptimizer {
    pub fn new(
        physics_config: PhysicsConfig,
        initial_spatial: SpatialComponent,
        initial_prop: PropulsionState,
        aircraft_config: FullAircraftConfig,
        condition: TrimCondition,
        settings: TrimSolverConfig,
    ) -> Self {
        // Calculate characteristic values for normalization
        let mass = aircraft_config.mass.mass;
        let gravity = physics_config.gravity.norm();
        let wingspan = aircraft_config.geometry.wing_span;

        Self {
            physics_config,
            initial_spatial,
            initial_prop,
            aircraft_config,
            condition,
            settings,
            characteristic_force: mass * gravity,
            characteristic_moment: mass * gravity * wingspan,
        }
    }

    fn create_virtual_physics(&self) -> (VirtualPhysics, Entity) {
        let mut virtual_physics = VirtualPhysics::new(&self.physics_config);
        let virtual_aircraft = virtual_physics.spawn_aircraft(
            &self.initial_spatial,
            &self.initial_prop,
            &self.aircraft_config,
        );
        (virtual_physics, virtual_aircraft)
    }

    pub fn calculate_residuals(&self, state: &TrimState) -> TrimResiduals {
        let (mut virtual_physics, virtual_aircraft) = self.create_virtual_physics();

        println!("State: {:?}", state);

        let control_surfaces = AircraftControlSurfaces {
            elevator: state.elevator,
            aileron: state.aileron,
            rudder: state.rudder,
            power_lever: state.power_lever,
        };

        let attitude = UnitQuaternion::from_euler_angles(state.phi, state.theta, 0.0);

        // Set velocities based on trim condition
        let (vx, vy, vz) = match self.condition {
            TrimCondition::StraightAndLevel { airspeed } => {
                let alpha = state.alpha;
                (airspeed * alpha.cos(), 0.0, -airspeed * alpha.sin())
            }
            TrimCondition::SteadyClimb { airspeed, gamma } => {
                let alpha = state.alpha;
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
                let alpha = state.alpha;
                let beta = state.beta;
                (
                    airspeed * alpha.cos() * beta.cos(),
                    airspeed * beta.sin(),
                    -airspeed * alpha.sin() * beta.cos(),
                )
            }
        };

        let velocity = Vector3::new(vx, vy, vz);

        virtual_physics.set_state(virtual_aircraft, &velocity, &attitude);
        virtual_physics.set_controls(virtual_aircraft, &control_surfaces);
        virtual_physics.run_steps(virtual_aircraft, 50);

        // Calculate forces at final state
        let (forces, moments) = virtual_physics.calculate_forces(virtual_aircraft);
        let (final_spatial, _final_controls) = virtual_physics.get_state(virtual_aircraft);

        TrimResiduals {
            forces: forces / self.characteristic_force,
            moments: moments / self.characteristic_moment,
            gamma_error: match self.condition {
                TrimCondition::SteadyClimb { gamma, .. } => {
                    let actual_gamma =
                        (-final_spatial.velocity.z / final_spatial.velocity.x).atan();
                    gamma - actual_gamma
                }
                _ => 0.0,
            },
            mu_error: match self.condition {
                TrimCondition::CoordinatedTurn { bank_angle, .. } => bank_angle - state.phi,
                _ => 0.0,
            },
        }
    }

    fn check_bounds(&self, state: &TrimState) -> bool {
        state.elevator >= self.settings.bounds.elevator_range.0
            && state.elevator <= self.settings.bounds.elevator_range.1
            && state.aileron >= self.settings.bounds.aileron_range.0
            && state.aileron <= self.settings.bounds.aileron_range.1
            && state.rudder >= self.settings.bounds.rudder_range.0
            && state.rudder <= self.settings.bounds.rudder_range.1
            && state.power_lever >= self.settings.bounds.throttle_range.0
            && state.power_lever <= self.settings.bounds.throttle_range.1
            && state.alpha >= self.settings.bounds.alpha_range.0
            && state.alpha <= self.settings.bounds.alpha_range.1
            && state.beta >= self.settings.bounds.beta_range.0
            && state.beta <= self.settings.bounds.beta_range.1
            && state.phi >= self.settings.bounds.phi_range.0
            && state.phi <= self.settings.bounds.phi_range.1
            && state.theta >= self.settings.bounds.theta_range.0
            && state.theta <= self.settings.bounds.theta_range.1
    }
}

pub struct TrimSolver {
    pub optimizer: TrimOptimizer,
    pub current_state: TrimState,
    pub best_state: TrimState,
    pub best_cost: f64,
    pub iteration: usize,
    use_gradient_refinement: bool,
    stage: OptimizationStage,
}

pub enum OptimizationStage {
    DirectSearch,  // Initial coarse optimization
    GradientBased, // Fine-tuning with LBFGS
}

impl TrimSolver {
    pub fn new(
        settings: TrimSolverConfig,
        initial_spatial_state: SpatialComponent,
        initial_prop_state: PropulsionState,
        aircraft_config: FullAircraftConfig,
        condition: TrimCondition,
        physics_config: &PhysicsConfig,
    ) -> Self {
        let optimizer = TrimOptimizer::new(
            physics_config.clone(),
            initial_spatial_state,
            initial_prop_state,
            aircraft_config,
            condition,
            settings,
        );

        Self {
            optimizer,
            current_state: TrimState::default(),
            best_state: TrimState::default(),
            best_cost: f64::MAX,
            iteration: 0,
            use_gradient_refinement: settings.use_gradient_refinement,
            stage: OptimizationStage::DirectSearch,
        }
    }

    pub fn initialize(&mut self, initial_guess: TrimState) {
        self.current_state = initial_guess;
        self.best_state = initial_guess;
        self.best_cost = f64::MAX;
        self.iteration = 0;
    }

    pub fn iterate(&mut self) -> Result<bool, Error> {
        match self.stage {
            OptimizationStage::DirectSearch => {
                let result = self.direct_search_step()?;

                if let Some(param) = &result.state.param {
                    let current_cost = result.state.cost;
                    let trim_state = TrimState::from_vector(param);

                    // Update best solution if better
                    if current_cost < self.best_cost {
                        self.best_cost = current_cost;
                        self.best_state = trim_state.clone();
                        self.current_state = trim_state;
                    }

                    // Switch to gradient-based if direct search converges or stalls
                    if self.iteration > 50 && current_cost < 1.0 && self.use_gradient_refinement {
                        self.stage = OptimizationStage::GradientBased;
                        println!(
                            "Switching to gradient-based optimization. Cost: {}",
                            current_cost
                        );
                    }
                }
            }
            OptimizationStage::GradientBased => {
                let result = self.gradient_step()?;

                if let Some(param) = &result.state.param {
                    let current_cost = result.state.cost;
                    let trim_state = TrimState::from_vector(param);

                    // Update best solution if better
                    if current_cost < self.best_cost {
                        self.best_cost = current_cost;
                        self.best_state = trim_state.clone();
                        self.current_state = trim_state;
                    }
                }
            }
        }
        self.iteration += 1;
        Ok(self.has_converged())
    }

    fn direct_search_step(
        &mut self,
    ) -> Result<
        OptimizationResult<
            TrimOptimizer,
            NelderMead<Vec<f64>, f64>,
            IterState<Vec<f64>, (), (), (), (), f64>,
        >,
        Error,
    > {
        let init_param = self.current_state.to_vector();
        let n = init_param.len();
        let mut simplex = Vec::with_capacity(n + 1);
        simplex.push(init_param.clone());

        // Create remaining vertices with small perturbations
        for i in 0..n {
            let mut vertex = init_param.clone();

            // Scale the perturbation based on parameter magnitude
            let perturbation = if vertex[i].abs() > 1e-10 {
                0.05 * vertex[i].abs() // 5% of current value
            } else {
                0.001 // Small constant for near-zero parameters
            };

            vertex[i] += perturbation;
            simplex.push(vertex);
        }

        let solver = NelderMead::new(simplex).with_sd_tolerance(1e-4)?;

        let optimizer = TrimOptimizer::new(
            self.optimizer.physics_config.clone(),
            self.optimizer.initial_spatial.clone(),
            self.optimizer.initial_prop.clone(),
            self.optimizer.aircraft_config.clone(),
            self.optimizer.condition,
            self.optimizer.settings.clone(),
        );

        Executor::new(optimizer, solver)
            .configure(|state| {
                state
                    .max_iters(100)
                    .target_cost(self.optimizer.settings.cost_tolerance)
            })
            .run()
    }

    fn gradient_step(
        &mut self,
    ) -> Result<
        OptimizationResult<
            TrimOptimizer,
            LBFGS<MoreThuenteLineSearch<Vec<f64>, Vec<f64>, f64>, Vec<f64>, Vec<f64>, f64>,
            IterState<Vec<f64>, Vec<f64>, (), (), (), f64>,
        >,
        Error,
    > {
        println!("Starting iteration {}", self.iteration);

        // Create optimizer from current state
        let optimizer = TrimOptimizer::new(
            self.optimizer.physics_config.clone(),
            self.optimizer.initial_spatial.clone(),
            self.optimizer.initial_prop.clone(),
            self.optimizer.aircraft_config.clone(),
            self.optimizer.condition,
            self.optimizer.settings.clone(),
        );

        // Convert current state to Vector
        let init_param = self.current_state.to_vector();
        let residuals = self.optimizer.calculate_residuals(&self.current_state);
        println!("Initial residuals: {:?}", residuals);

        // Print initial gradients
        if let Ok(grad) = optimizer.gradient(&init_param) {
            println!("Initial gradients: {:?}", grad);
        }

        // Set up line search
        let linesearch = MoreThuenteLineSearch::new()
            .with_c(1e-4, 0.5)
            .expect("Failed to configure line search");

        // Set up LBFGS solver
        let solver = LBFGS::new(linesearch, 7);

        Executor::new(optimizer, solver)
            .configure(|state| {
                state
                    .param(init_param)
                    .max_iters(2000)
                    .target_cost(self.optimizer.settings.cost_tolerance)
            })
            .run()
    }

    pub fn has_converged(&self) -> bool {
        self.best_cost < self.optimizer.settings.cost_tolerance
    }

    pub fn get_best_solution(&mut self) -> TrimResult {
        TrimResult {
            state: self.best_state,
            converged: self.has_converged(),
            cost: self.best_cost,
            iterations: self.iteration,
            residuals: self.optimizer.calculate_residuals(&mut self.best_state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Matrix3, Vector3};

    use crate::{
        components::{
            AirData, AircraftAeroCoefficients, AircraftGeometry, AircraftType, DragCoefficients,
            FullAircraftConfig, LiftCoefficients, MassModel, NeedsTrim, PitchCoefficients,
            PowerplantConfig, PowerplantState, PropulsionConfig, PropulsionState, SpatialComponent,
            TrimBounds,
        },
        systems::trim_aircraft_system,
    };

    // Helper function to create a basic test app with required resources
    fn create_test_app() -> App {
        let mut app = App::new();

        // Add required plugins and systems
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, trim_aircraft_system);

        // Add required resources
        app.insert_resource(PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        });

        app.insert_resource(TrimSolverConfig {
            max_iterations: 100,
            cost_tolerance: 1e-3,
            use_gradient_refinement: true,
            bounds: TrimBounds::default(),
        });

        app
    }

    // Helper to create a basic aircraft entity
    fn create_test_aircraft_config() -> FullAircraftConfig {
        // Create a simple test aircraft with reasonable parameters
        let mass = 1000.0; // 1000 kg
        let wing_area = 16.0; // 16 m^2
        let wing_span = 10.0; // 10 m
        let mac = 1.6; // 1.6 m mean aerodynamic chord

        // Create basic configuration
        FullAircraftConfig {
            name: "test_aircraft".to_string(),
            ac_type: AircraftType::Custom("TestAircraft".to_string()),
            mass: MassModel {
                mass,
                inertia: Matrix3::from_diagonal(&Vector3::new(1000.0, 2000.0, 1500.0)),
                inertia_inv: Matrix3::from_diagonal(&Vector3::new(
                    1.0 / 1000.0,
                    1.0 / 2000.0,
                    1.0 / 1500.0,
                )),
            },
            geometry: AircraftGeometry {
                wing_area,
                wing_span,
                mac,
            },
            aero_coef: AircraftAeroCoefficients {
                // Simple but physically plausible coefficients
                lift: LiftCoefficients {
                    c_l_0: 0.2,
                    c_l_alpha: 5.0,
                    ..Default::default()
                },
                drag: DragCoefficients {
                    c_d_0: 0.02,
                    c_d_alpha2: 0.1,
                    ..Default::default()
                },
                pitch: PitchCoefficients {
                    c_m_0: 0.0,
                    c_m_alpha: -1.0,
                    c_m_q: -10.0,
                    c_m_deltae: -1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            propulsion: PropulsionConfig {
                engines: vec![PowerplantConfig {
                    name: "engine1".to_string(),
                    max_thrust: 5000.0,
                    min_thrust: 0.0,
                    position: Vector3::new(0.0, 0.0, 0.0),
                    orientation: Vector3::new(1.0, 0.0, 0.0),
                    tsfc: 0.0001,
                    spool_up_time: 1.0,
                    spool_down_time: 1.0,
                }],
            },
            start_config: Default::default(),
            task_config: Default::default(),
        }
    }

    fn spawn_test_aircraft(app: &mut App) -> Entity {
        let initial_spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0),
            velocity: Vector3::new(100.0, 0.0, 0.0), // Initial forward velocity
            attitude: UnitQuaternion::from_euler_angles(0.0, 0.05, 0.0), // Small initial pitch
            angular_velocity: Vector3::zeros(),
        };

        let initial_air_data = AirData {
            true_airspeed: 100.0,
            alpha: 0.05,
            beta: 0.0,
            dynamic_pressure: 0.5 * 1.225 * 100.0 * 100.0,
            density: 1.225,
            relative_velocity: Vector3::new(100.0, 0.0, 0.0),
            wind_velocity: Vector3::zeros(),
        };

        let initial_controls = AircraftControlSurfaces {
            elevator: 0.0,
            aileron: 0.0,
            rudder: 0.0,
            power_lever: 0.5,
        };

        let power_plant = PowerplantState {
            power_lever: 0.5,
            thrust_fraction: 0.5,
            fuel_flow: 0.0,
            running: true,
        };

        let initial_propulsion = PropulsionState {
            engine_states: vec![power_plant],
        };

        app.world_mut()
            .spawn((
                initial_spatial,
                initial_controls,
                initial_propulsion,
                initial_air_data,
                create_test_aircraft_config(),
                NeedsTrim {
                    condition: TrimCondition::StraightAndLevel { airspeed: 100.0 },
                    solver: None,
                },
            ))
            .id()
    }

    #[test]
    fn test_solver_initialization() {
        let mut app = create_test_app();
        let aircraft_entity = spawn_test_aircraft(&mut app);

        // First update to initialize
        app.update();

        // Check that initialization occurred
        let needs_trim = app
            .world()
            .get::<NeedsTrim>(aircraft_entity)
            .expect("NeedsTrim component should exist after initialization");

        assert!(needs_trim.solver.is_some(), "Solver should be initialized");

        if let Some(ref solver) = needs_trim.solver {
            assert_eq!(solver.iteration, 0, "Initial iteration should be 0");
            assert!(!solver.has_converged(), "Should not be converged initially");
        }
    }

    #[test]
    fn test_basic_trim_convergence() {
        let mut app = create_test_app();

        {
            let mut solver_config = app.world_mut().resource_mut::<TrimSolverConfig>();
            solver_config.cost_tolerance = 1e-2;
            solver_config.max_iterations = 100;

            // Add more reasonable bounds
            solver_config.bounds = TrimBounds {
                elevator_range: (-0.5, 0.5),
                aileron_range: (-0.3, 0.3),
                rudder_range: (-0.3, 0.3),
                throttle_range: (0.1, 0.9),
                alpha_range: (-0.2, 0.2),
                beta_range: (-0.1, 0.1),
                phi_range: (-0.2, 0.2),
                theta_range: (-0.2, 0.2),
            };
        }

        let aircraft_entity = spawn_test_aircraft(&mut app);

        if let Some(mut spatial) = app.world_mut().get_mut::<SpatialComponent>(aircraft_entity) {
            spatial.velocity = Vector3::new(50.0, 0.0, 0.0);
            spatial.attitude = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
        }

        let mut last_cost = f64::INFINITY;
        let mut stall_counter = 0;

        for i in 0..50 {
            app.update();

            if let Some(needs_trim) = app.world().get::<NeedsTrim>(aircraft_entity) {
                if let Some(ref solver) = needs_trim.solver {
                    let current_cost = solver.best_cost;
                    println!("Iteration {}: Cost = {}", i, current_cost);

                    // Check for reasonable cost values
                    if !current_cost.is_finite() {
                        panic!("Cost became non-finite at iteration {}", i);
                    }

                    // Check if we're making progress
                    if (last_cost - current_cost).abs() < 1e-6 {
                        stall_counter += 1;
                        if stall_counter > 5 {
                            println!("Optimization stalled - not making progress");
                            break;
                        }
                    } else {
                        stall_counter = 0;
                    }

                    // Success condition
                    if current_cost < 1e-2 {
                        println!("Successfully converged at iteration {}", i);
                        return;
                    }

                    last_cost = current_cost;
                }
            } else {
                // NeedsTrim component removed - should be converged
                return;
            }
        }

        // If we get here, print final state for debugging
        if let Some(needs_trim) = app.world().get::<NeedsTrim>(aircraft_entity) {
            if let Some(ref solver) = needs_trim.solver {
                println!("Final state: {:?}", solver.current_state);
                println!(
                    "Final residuals: {:?}",
                    solver.optimizer.calculate_residuals(&solver.current_state)
                );
            }
        }

        panic!("Failed to converge within iteration limit");
    }

    #[test]
    fn test_gradient_calculation() {
        let physics_config = PhysicsConfig::default();
        let initial_spatial = SpatialComponent::default();
        let initial_prop = PropulsionState::default();
        let aircraft_config = create_test_aircraft_config();
        let condition = TrimCondition::StraightAndLevel { airspeed: 100.0 };
        let settings = TrimSolverConfig::default();

        let optimizer = TrimOptimizer::new(
            physics_config,
            initial_spatial,
            initial_prop,
            aircraft_config,
            condition,
            settings,
        );

        let state = TrimState::default();
        let param = state.to_vector();

        let gradient = optimizer.gradient(&param).unwrap();

        // Verify gradient is not all zeros
        assert!(
            gradient.iter().any(|&x| x.abs() > 1e-6),
            "Gradient should not be all zeros: {:?}",
            gradient
        );

        // Verify gradient magnitude is reasonable
        let gradient_norm = gradient.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(
            gradient_norm < 1e3,
            "Gradient magnitude should be reasonable: {}",
            gradient_norm
        );
    }

    #[test]
    fn test_residuals_calculation() {
        let physics_config = PhysicsConfig::default();
        let initial_spatial = SpatialComponent::default();
        let initial_prop = PropulsionState::default();
        let aircraft_config = create_test_aircraft_config();
        let condition = TrimCondition::StraightAndLevel { airspeed: 100.0 };
        let settings = TrimSolverConfig::default();

        let optimizer = TrimOptimizer::new(
            physics_config,
            initial_spatial,
            initial_prop,
            aircraft_config,
            condition,
            settings,
        );

        let state = TrimState::default();
        let residuals = optimizer.calculate_residuals(&state);

        println!("Forces: {:?}", residuals.forces);
        println!("Moments: {:?}", residuals.moments);

        // Check that residuals are finite
        assert!(
            residuals.forces.iter().all(|x| x.is_finite()),
            "Forces should be finite"
        );
        assert!(
            residuals.moments.iter().all(|x| x.is_finite()),
            "Moments should be finite"
        );
    }

    #[test]
    fn test_residuals_determinism() {
        // Create a basic optimizer setup
        let physics_config = PhysicsConfig {
            timestep: 0.01,
            gravity: Vector3::new(0.0, 0.0, 9.81),
            max_velocity: 200.0,
            max_angular_velocity: 10.0,
        };

        let test_state = TrimState {
            elevator: 0.1,
            aileron: 0.0,
            rudder: 0.0,
            power_lever: 0.5,
            alpha: 0.05,
            beta: 0.0,
            phi: 0.0,
            theta: 0.05,
        };

        let initial_spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0),
            velocity: Vector3::new(100.0, 0.0, 0.0),
            attitude: UnitQuaternion::from_euler_angles(0.0, 0.05, 0.0),
            angular_velocity: Vector3::zeros(),
        };

        let initial_prop = PropulsionState::default();
        let aircraft_config = create_test_aircraft_config();
        let condition = TrimCondition::StraightAndLevel { airspeed: 100.0 };
        let settings = TrimSolverConfig::default();

        let optimizer = TrimOptimizer::new(
            physics_config,
            initial_spatial,
            initial_prop,
            aircraft_config,
            condition,
            settings,
        );

        // Calculate residuals multiple times
        let residuals1 = optimizer.calculate_residuals(&test_state);
        let residuals2 = optimizer.calculate_residuals(&test_state);
        let residuals3 = optimizer.calculate_residuals(&test_state);

        // Check forces are identical
        assert!(
            (residuals1.forces - residuals2.forces).norm() < 1e-10,
            "Forces differ between first and second calculation:\n{:?}\n{:?}",
            residuals1.forces,
            residuals2.forces
        );
        assert!(
            (residuals2.forces - residuals3.forces).norm() < 1e-10,
            "Forces differ between second and third calculation"
        );

        // Check moments are identical
        assert!(
            (residuals1.moments - residuals2.moments).norm() < 1e-10,
            "Moments differ between first and second calculation:\n{:?}\n{:?}",
            residuals1.moments,
            residuals2.moments
        );
        assert!(
            (residuals2.moments - residuals3.moments).norm() < 1e-10,
            "Moments differ between second and third calculation"
        );

        // Check gamma and mu errors
        assert!(
            (residuals1.gamma_error - residuals2.gamma_error).abs() < 1e-10,
            "Gamma error differs between calculations"
        );
        assert!(
            (residuals1.mu_error - residuals2.mu_error).abs() < 1e-10,
            "Mu error differs between calculations"
        );

        // Also check cost function determinism
        let param = test_state.to_vector();
        let cost1 = optimizer.cost(&param).unwrap();
        let cost2 = optimizer.cost(&param).unwrap();
        let cost3 = optimizer.cost(&param).unwrap();

        assert!(
            (cost1 - cost2).abs() < 1e-10,
            "Cost differs between first and second calculation: {} vs {}",
            cost1,
            cost2
        );
        assert!(
            (cost2 - cost3).abs() < 1e-10,
            "Cost differs between second and third calculation"
        );
    }

    #[test]
    fn test_solver_convergence() {
        let mut app = create_test_app();
        let aircraft_entity = spawn_test_aircraft(&mut app);

        app.add_systems(Update, trim_aircraft_system);

        // Run until convergence or max iterations
        let max_updates = 500;
        let mut converged = false;

        for i in 0..max_updates {
            println!("iter: {}", i);
            app.update();

            // Check if component was removed (indicating convergence)
            if app.world().get::<NeedsTrim>(aircraft_entity).is_none() {
                converged = true;
                println!("Converged after {} iterations", i);
                break;
            }
        }

        assert!(converged, "Solver should converge within max iterations");

        // Verify final state
        let spatial = app
            .world()
            .get::<SpatialComponent>(aircraft_entity)
            .unwrap();
        let air_data = app.world().get::<AirData>(aircraft_entity).unwrap();

        // Check that forces are balanced
        assert!(
            air_data.true_airspeed > 98.0 && air_data.true_airspeed < 102.0,
            "Airspeed should be near target: {}",
            air_data.true_airspeed
        );

        let (roll, pitch, _) = spatial.attitude.euler_angles();
        assert!(roll.abs() < 0.1, "Roll angle should be near zero: {}", roll);
        assert!(
            pitch.abs() < 0.2,
            "Pitch angle should be reasonable: {}",
            pitch
        );
    }

    #[test]
    fn test_solver_bounds() {
        let mut app = create_test_app();
        let aircraft_entity = spawn_test_aircraft(&mut app);

        app.update();

        let needs_trim = app
            .world()
            .get::<NeedsTrim>(aircraft_entity)
            .expect("NeedsTrim component should exist");
        let solver = needs_trim.solver.as_ref().unwrap();

        // Get solver state
        let state = solver.current_state;
        let bounds = &solver.optimizer.settings.bounds;

        assert!(
            state.elevator >= bounds.elevator_range.0 && state.elevator <= bounds.elevator_range.1,
            "Elevator out of bounds: {}",
            state.elevator
        );

        assert!(
            state.power_lever >= bounds.throttle_range.0
                && state.power_lever <= bounds.throttle_range.1,
            "Throttle out of bounds: {}",
            state.power_lever
        );

        assert!(
            state.alpha >= bounds.alpha_range.0 && state.alpha <= bounds.alpha_range.1,
            "Alpha out of bounds: {}",
            state.alpha
        );
    }

    #[test]
    fn test_invalid_initial_conditions() {
        let mut app = create_test_app();

        // Spawn aircraft with extreme initial conditions
        let aircraft_entity = app
            .world_mut()
            .spawn((
                SpatialComponent {
                    velocity: Vector3::new(1000.0, 0.0, 0.0), // Way too fast
                    ..default()
                },
                AircraftControlSurfaces::default(),
                PropulsionState::default(),
                AirData::default(),
                create_test_aircraft_config(),
                NeedsTrim {
                    condition: TrimCondition::StraightAndLevel { airspeed: 100.0 },
                    solver: None,
                },
            ))
            .id();

        // Run system and verify it handles extreme conditions gracefully
        app.update();

        let needs_trim = app
            .world()
            .get::<NeedsTrim>(aircraft_entity)
            .expect("NeedsTrim component should exist");

        assert!(
            needs_trim.solver.is_some(),
            "Solver should initialize even with invalid conditions"
        );
    }
}
