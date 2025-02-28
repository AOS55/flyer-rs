use argmin::{
    core::{CostFunction, Error, Executor, Gradient, IterState, OptimizationResult},
    solver::{linesearch::MoreThuenteLineSearch, neldermead::NelderMead, quasinewton::LBFGS},
};
use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    components::{
        AircraftControlSurfaces, FullAircraftConfig, LateralResiduals, LongitudinalResiduals,
        LongitudinalTrimState, PropulsionState, SpatialComponent, TrimCondition, TrimResiduals,
        TrimResult, TrimSolverConfig, TrimState,
    },
    resources::PhysicsConfig,
    systems::VirtualPhysics,
};

#[derive(Debug, Clone)]
pub struct TrimOptimizer {
    physics_config: PhysicsConfig,
    initial_spatial: SpatialComponent,
    initial_prop: PropulsionState,
    aircraft_config: FullAircraftConfig,
    condition: TrimCondition,
    settings: TrimSolverConfig,
    characteristic_force: f64,
    characteristic_moment: f64,
    mode: TrimMode,
}

#[derive(Debug, Clone, Copy)]
enum TrimMode {
    LongitudinalOnly,
    Sequential, // Longitudinal then lateral
    Combined,   // Both together for turning flight
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
        let mass = aircraft_config.mass.mass;
        let gravity = physics_config.gravity.norm();
        let wingspan = aircraft_config.geometry.wing_span;

        let mode = match condition {
            TrimCondition::StraightAndLevel { .. } => TrimMode::LongitudinalOnly,
            TrimCondition::SteadyClimb { .. } => TrimMode::LongitudinalOnly,
            TrimCondition::CoordinatedTurn { .. } => TrimMode::Sequential,
        };

        Self {
            physics_config,
            initial_spatial,
            initial_prop,
            aircraft_config,
            condition,
            settings,
            characteristic_force: mass * gravity,
            characteristic_moment: mass * gravity * wingspan,
            mode,
        }
    }

    fn calculate_constraint_penalty(&self, value: f64, range: (f64, f64), weight: f64) -> f64 {
        let (min, max) = range;
        let below_min = if value < min {
            (min - value).powi(4)
        } else {
            0.0
        };
        let above_max = if value > max {
            (value - max).powi(4)
        } else {
            0.0
        };
        weight * (below_min + above_max)
    }

    fn calculate_longitudinal_residuals(
        &self,
        state: &LongitudinalTrimState,
    ) -> LongitudinalResiduals {
        let (mut virtual_physics, virtual_aircraft) = self.create_virtual_physics();

        // Debug input state
        println!("\n=== Residuals Calculation Debug ===");
        println!("Input state:");
        println!("  Alpha: {:.1}°", state.alpha.to_degrees());
        println!("  Theta: {:.1}°", state.theta.to_degrees());
        println!("  Elevator: {:.3}", state.elevator);
        println!("  Throttle: {:.3}", state.power_lever);

        // Calculate and print velocities
        let (vx, vy, vz) = match self.condition {
            TrimCondition::StraightAndLevel { airspeed } => {
                let alpha = state.alpha;
                let vx = airspeed * alpha.cos();
                let vz = -airspeed * alpha.sin();
                println!("\nCalculated velocities:");
                println!("  Vx: {:.1} m/s", vx);
                println!("  Vz: {:.1} m/s", vz);
                println!("  Airspeed: {:.1} m/s", (vx * vx + vz * vz).sqrt());
                (vx, 0.0, vz)
            }
            _ => panic!("Invalid trim condition for longitudinal residuals"),
        };

        // Set the state and run simulation
        let velocity = Vector3::new(vx, vy, vz);
        let attitude = UnitQuaternion::from_euler_angles(0.0, state.theta, 0.0);

        virtual_physics.set_state(virtual_aircraft, &velocity, &attitude);
        virtual_physics.set_controls(
            virtual_aircraft,
            &AircraftControlSurfaces {
                elevator: state.elevator,
                aileron: 0.0,
                rudder: 0.0,
                power_lever: state.power_lever,
            },
        );

        // Print forces before settling
        let (initial_forces, initial_moments) = virtual_physics.calculate_forces(virtual_aircraft);
        println!("\nInitial forces/moments:");
        println!("  Forces: {:?}", initial_forces);
        println!("  Moments: {:?}", initial_moments);

        // Run simulation to settle
        virtual_physics.run_steps(virtual_aircraft, 6000);

        // Calculate average forces
        let num_steps = 2000;
        let mut force_history = Vec::with_capacity(num_steps);
        let mut moment_history = Vec::with_capacity(num_steps);

        for _ in 0..num_steps {
            virtual_physics.run_steps(virtual_aircraft, 1);
            let (forces, moments) = virtual_physics.calculate_forces(virtual_aircraft);
            force_history.push(forces);
            moment_history.push(moments);
        }

        let mean_forces = force_history.iter().sum::<Vector3<f64>>() / num_steps as f64;
        let mean_moments = moment_history.iter().sum::<Vector3<f64>>() / num_steps as f64;

        // Print final state
        let (final_spatial, _) = virtual_physics.get_state(virtual_aircraft);
        println!("\nFinal state:");
        println!("  Velocity: {:?}", final_spatial.velocity);
        println!("  Forces: {:?}", mean_forces);
        println!("  Moments: {:?}", mean_moments);

        // Calculate and normalize residuals
        let residuals = LongitudinalResiduals {
            vertical_force: mean_forces.z / self.characteristic_force,
            horizontal_force: mean_forces.x / self.characteristic_force,
            pitch_moment: mean_moments.y / self.characteristic_moment,
            gamma_error: state.alpha - state.theta, // Assumes S+L
        };

        println!("\nResiduals:");
        println!("  Vertical force: {:.3}", residuals.vertical_force);
        println!("  Horizontal force: {:.3}", residuals.horizontal_force);
        println!("  Pitch moment: {:.3}", residuals.pitch_moment);
        println!("  Gamma error: {:.1}°", residuals.gamma_error.to_degrees());

        residuals
    }

    // fn calculate_longitudinal_residuals(
    //     &self,
    //     state: &LongitudinalTrimState,
    // ) -> LongitudinalResiduals {
    //     let (mut virtual_physics, virtual_aircraft) = self.create_virtual_physics();

    //     // Set controls (only longitudinal)
    //     let control_surfaces = AircraftControlSurfaces {
    //         elevator: state.elevator,
    //         aileron: 0.0,
    //         rudder: 0.0,
    //         power_lever: state.power_lever,
    //     };

    //     // Set attitude (only pitch)
    //     let attitude = UnitQuaternion::from_euler_angles(0.0, state.theta, 0.0);

    //     // Calculate velocities based on condition
    //     let (vx, vy, vz) = match self.condition {
    //         TrimCondition::StraightAndLevel { airspeed } => {
    //             let alpha = state.alpha;
    //             (airspeed * alpha.cos(), 0.0, -airspeed * alpha.sin())
    //         }
    //         TrimCondition::SteadyClimb { airspeed, gamma } => {
    //             let alpha = state.alpha;
    //             (
    //                 airspeed * (alpha - gamma).cos(),
    //                 0.0,
    //                 -airspeed * (alpha - gamma).sin(),
    //             )
    //         }
    //         _ => panic!("Invalid trim condition for longitudinal residuals"),
    //     };

    //     let velocity = Vector3::new(vx, vy, vz);

    //     virtual_physics.set_state(virtual_aircraft, &velocity, &attitude);
    //     virtual_physics.set_controls(virtual_aircraft, &control_surfaces);

    //     // Run simulation to settle
    //     virtual_physics.run_steps(virtual_aircraft, 200);

    //     // Calculate average forces over several steps
    //     let num_steps = 50;
    //     let mut force_history = Vec::with_capacity(num_steps);
    //     let mut moment_history = Vec::with_capacity(num_steps);

    //     for _ in 0..num_steps {
    //         virtual_physics.run_steps(virtual_aircraft, 1);
    //         let (forces, moments) = virtual_physics.calculate_forces(virtual_aircraft);
    //         force_history.push(forces);
    //         moment_history.push(moments);
    //     }

    //     let mean_forces = force_history.iter().sum::<Vector3<f64>>() / num_steps as f64;
    //     let mean_moments = moment_history.iter().sum::<Vector3<f64>>() / num_steps as f64;

    //     // // Debug prints
    //     // println!("\n=== Longitudinal State Verification ===");
    //     // println!("Controls:");
    //     // println!("  Elevator: {:.3} (should be -0.5 to 0.5)", state.elevator);
    //     // println!(
    //     //     "  Throttle: {:.3} (should be 0.0 to 1.0)",
    //     //     state.power_lever
    //     // );
    //     // println!("\nAttitude:");
    //     // println!("  Alpha: {:.1}° (expect 2-10°)", state.alpha.to_degrees());
    //     // println!("  Theta: {:.1}° (should ≈ alpha)", state.theta.to_degrees());
    //     // println!("\nForces (normalized):");
    //     // println!(
    //     //     "  Vertical: {:.3}",
    //     //     mean_forces.z / self.characteristic_force
    //     // );
    //     // println!(
    //     //     "  Forward: {:.3}",
    //     //     mean_forces.x / self.characteristic_force
    //     // );
    //     // println!(
    //     //     "  Pitch moment: {:.3}",
    //     //     mean_moments.y / self.characteristic_moment
    //     // );

    //     LongitudinalResiduals {
    //         vertical_force: mean_forces.z / self.characteristic_force,
    //         horizontal_force: mean_forces.x / self.characteristic_force,
    //         pitch_moment: mean_moments.y / self.characteristic_moment,
    //         gamma_error: match self.condition {
    //             TrimCondition::SteadyClimb { gamma, .. } => {
    //                 let actual_gamma = (-vz / vx).atan();
    //                 gamma - actual_gamma
    //             }
    //             _ => 0.0,
    //         },
    //     }
    // }

    fn create_virtual_physics(&self) -> (VirtualPhysics, Entity) {
        let mut virtual_physics = VirtualPhysics::new(&self.physics_config);
        let virtual_aircraft = virtual_physics.spawn_aircraft(
            &self.initial_spatial,
            &self.initial_prop,
            &self.aircraft_config,
        );
        (virtual_physics, virtual_aircraft)
    }
}

impl CostFunction for TrimOptimizer {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        match self.mode {
            TrimMode::LongitudinalOnly => {
                let state = LongitudinalTrimState::from_vector(param);
                let residuals = self.calculate_longitudinal_residuals(&state);

                // Weight the physically meaningful components
                let residual_cost = 10.0 * residuals.vertical_force.powi(2)      // Priority on lift balance
                    + 5.0 * residuals.horizontal_force.powi(2)     // Thrust/drag secondary
                    + 2.0 * residuals.pitch_moment.powi(2)         // Moment balance
                    + 10.0 * residuals.gamma_error.powi(2); // Flight path tracking

                // Constraint penalties
                let bounds = &self.settings.longitudinal_bounds;
                let constraint_cost = self.calculate_constraint_penalty(
                        state.elevator,
                        bounds.elevator_range,
                        100.0
                    ) +
                    self.calculate_constraint_penalty(
                        state.power_lever,
                        bounds.throttle_range,
                        100.0
                    ) +
                    self.calculate_constraint_penalty(
                        state.alpha,
                        bounds.alpha_range,
                        200.0
                    ) +
                    // Theta constraints (less critical but still important)
                    self.calculate_constraint_penalty(
                        state.theta,
                        bounds.theta_range,
                        150.0
                    );

                let total_cost = residual_cost + constraint_cost;
                Ok(total_cost)
            }
            _ => todo!("Implement lateral and combined modes"),
        }
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

        match self.mode {
            TrimMode::LongitudinalOnly => {
                // Calculate gradients for longitudinal parameters
                for i in 0..n {
                    // Central differences
                    let mut param_plus = param.clone();
                    let mut param_minus = param.clone();
                    param_plus[i] += eps;
                    param_minus[i] -= eps;

                    let f_plus = self.cost(&param_plus)?;
                    let f_minus = self.cost(&param_minus)?;

                    if f_plus.is_finite() && f_minus.is_finite() {
                        grad[i] = (f_plus - f_minus) / (2.0 * eps);
                        // Scale large gradients
                        if grad[i].abs() > 1.0 {
                            grad[i] = grad[i].signum() * grad[i].abs().sqrt();
                        }
                    }
                }

                println!("Gradients: {:?}", grad);
            }
            _ => todo!("Implement lateral and combined mode gradients"),
        }

        Ok(grad)
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
    DirectSearch,
    GradientBased,
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
            settings.clone(),
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
                let param = match self.optimizer.mode {
                    TrimMode::LongitudinalOnly => self.current_state.longitudinal.to_vector(),
                    _ => todo!("Implement lateral and combined modes"),
                };

                let result = self.direct_search_step(&param)?;

                if let Some(param) = &result.state.param {
                    let current_cost = result.state.cost;

                    match self.optimizer.mode {
                        TrimMode::LongitudinalOnly => {
                            let long_state = LongitudinalTrimState::from_vector(param);
                            let mut new_state = self.current_state;
                            new_state.longitudinal = long_state;

                            if current_cost < self.best_cost {
                                self.best_cost = current_cost;
                                self.best_state = new_state;
                            }
                            self.current_state = new_state;
                        }
                        _ => todo!("Implement lateral and combined modes"),
                    }

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
                let param = match self.optimizer.mode {
                    TrimMode::LongitudinalOnly => self.current_state.longitudinal.to_vector(),
                    _ => todo!("Implement lateral and combined modes"),
                };

                let result = self.gradient_step(&param)?;

                if let Some(param) = &result.state.param {
                    let current_cost = result.state.cost;

                    match self.optimizer.mode {
                        TrimMode::LongitudinalOnly => {
                            let long_state = LongitudinalTrimState::from_vector(param);
                            let mut new_state = self.current_state;
                            new_state.longitudinal = long_state;

                            if current_cost < self.best_cost {
                                self.best_cost = current_cost;
                                self.best_state = new_state;
                            }
                            self.current_state = new_state;
                        }
                        _ => todo!("Implement lateral and combined modes"),
                    }
                }
            }
        }
        self.iteration += 1;
        Ok(self.has_converged())
    }

    fn direct_search_step(
        &mut self,
        init_param: &[f64],
    ) -> Result<
        OptimizationResult<
            TrimOptimizer,
            NelderMead<Vec<f64>, f64>,
            IterState<Vec<f64>, (), (), (), (), f64>,
        >,
        Error,
    > {
        let n = init_param.len();
        let mut simplex = Vec::with_capacity(n + 1);
        simplex.push(init_param.to_vec());

        for i in 0..n {
            let mut vertex = init_param.to_vec();
            let perturbation = if vertex[i].abs() > 1e-10 {
                0.05 * vertex[i].abs()
            } else {
                0.001
            };
            vertex[i] += perturbation;
            simplex.push(vertex);
        }

        let solver = NelderMead::new(simplex).with_sd_tolerance(1e-4)?;

        Executor::new(self.optimizer.clone(), solver)
            .configure(|state| {
                state
                    .max_iters(100)
                    .target_cost(self.optimizer.settings.cost_tolerance)
            })
            .run()
    }

    fn gradient_step(
        &mut self,
        init_param: &[f64],
    ) -> Result<
        OptimizationResult<
            TrimOptimizer,
            LBFGS<MoreThuenteLineSearch<Vec<f64>, Vec<f64>, f64>, Vec<f64>, Vec<f64>, f64>,
            IterState<Vec<f64>, Vec<f64>, (), (), (), f64>,
        >,
        Error,
    > {
        println!("Starting iteration {}", self.iteration);

        let linesearch = MoreThuenteLineSearch::new()
            .with_c(1e-4, 0.5)
            .expect("Failed to configure line search");

        let solver = LBFGS::new(linesearch, 7);

        Executor::new(self.optimizer.clone(), solver)
            .configure(|state| {
                state
                    .param(init_param.to_vec())
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
            residuals: TrimResiduals {
                longitudinal: self
                    .optimizer
                    .calculate_longitudinal_residuals(&self.best_state.longitudinal),
                lateral: LateralResiduals::default(), // TODO: Implement when adding lateral trim
            },
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
            FullAircraftConfig, LateralBounds, LiftCoefficients, LongitudinalBounds, MassModel,
            NeedsTrim, PitchCoefficients, PowerplantConfig, PowerplantState, PropulsionConfig,
            PropulsionState, SpatialComponent, TrimBounds, TrimStage,
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
            lateral_bounds: LateralBounds::default(),
            longitudinal_bounds: LongitudinalBounds::default(),
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
                    stage: TrimStage::Longitudinal,
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
            solver_config.longitudinal_bounds = LongitudinalBounds {
                elevator_range: (-0.5, 0.5),
                throttle_range: (0.1, 0.9),
                alpha_range: (-0.2, 0.2),
                theta_range: (-0.2, 0.2),
            };
            solver_config.lateral_bounds = LateralBounds {
                aileron_range: (-0.3, 0.3),
                rudder_range: (-0.3, 0.3),
                beta_range: (-0.1, 0.1),
                phi_range: (-0.2, 0.2),
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
                    solver
                        .optimizer
                        .calculate_longitudinal_residuals(&solver.current_state.longitudinal)
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

        let state = LongitudinalTrimState::default();
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
    fn test_longitudinal_residuals_calculation() {
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

        let state = LongitudinalTrimState::default();
        let residuals = optimizer.calculate_longitudinal_residuals(&state);

        println!(
            "Forces: {:?}, {:?}",
            residuals.horizontal_force, residuals.vertical_force
        );
        println!("Moments: {:?}", residuals.pitch_moment);

        // Check that residuals are finite
        assert!(
            residuals.horizontal_force.is_finite(),
            "Forces should be finite"
        );
        assert!(
            residuals.pitch_moment.is_finite(),
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

        let test_state = LongitudinalTrimState {
            elevator: 0.1,
            power_lever: 0.5,
            alpha: 0.05,
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
        let residuals1 = optimizer.calculate_longitudinal_residuals(&test_state);
        let residuals2 = optimizer.calculate_longitudinal_residuals(&test_state);
        let residuals3 = optimizer.calculate_longitudinal_residuals(&test_state);

        // Check forces are identical
        assert!(
            (residuals1.horizontal_force - residuals2.horizontal_force) < 1e-10,
            "Forces differ between first and second calculation:\n{:?}\n{:?}",
            residuals1.horizontal_force,
            residuals2.horizontal_force
        );
        assert!(
            (residuals2.horizontal_force - residuals3.horizontal_force) < 1e-10,
            "Forces differ between second and third calculation"
        );

        // Check moments are identical
        assert!(
            (residuals1.pitch_moment - residuals2.pitch_moment) < 1e-10,
            "Moments differ between first and second calculation:\n{:?}\n{:?}",
            residuals1.pitch_moment,
            residuals2.pitch_moment
        );
        assert!(
            (residuals2.pitch_moment - residuals3.pitch_moment) < 1e-10,
            "Moments differ between second and third calculation"
        );

        // Check gamma and mu errors
        assert!(
            (residuals1.gamma_error - residuals2.gamma_error).abs() < 1e-10,
            "Gamma error differs between calculations"
        );
        assert!(
            (residuals1.vertical_force - residuals2.vertical_force).abs() < 1e-10,
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

    // #[test]
    // fn test_solver_bounds() {
    //     let mut app = create_test_app();
    //     let aircraft_entity = spawn_test_aircraft(&mut app);

    //     app.update();

    //     let needs_trim = app
    //         .world()
    //         .get::<NeedsTrim>(aircraft_entity)
    //         .expect("NeedsTrim component should exist");
    //     let solver = needs_trim.solver.as_ref().unwrap();

    //     // Get solver state
    //     let state = solver.current_state;
    //     let bounds = &solver.optimizer.settings.bounds;

    //     assert!(
    //         state.elevator >= bounds.elevator_range.0 && state.elevator <= bounds.elevator_range.1,
    //         "Elevator out of bounds: {}",
    //         state.elevator
    //     );

    //     assert!(
    //         state.power_lever >= bounds.throttle_range.0
    //             && state.power_lever <= bounds.throttle_range.1,
    //         "Throttle out of bounds: {}",
    //         state.power_lever
    //     );

    //     assert!(
    //         state.alpha >= bounds.alpha_range.0 && state.alpha <= bounds.alpha_range.1,
    //         "Alpha out of bounds: {}",
    //         state.alpha
    //     );
    // }

    // #[test]
    // fn test_invalid_initial_conditions() {
    //     let mut app = create_test_app();

    //     // Spawn aircraft with extreme initial conditions
    //     let aircraft_entity = app
    //         .world_mut()
    //         .spawn((
    //             SpatialComponent {
    //                 velocity: Vector3::new(1000.0, 0.0, 0.0), // Way too fast
    //                 ..default()
    //             },
    //             AircraftControlSurfaces::default(),
    //             PropulsionState::default(),
    //             AirData::default(),
    //             create_test_aircraft_config(),
    //             NeedsTrim {
    //                 condition: TrimCondition::StraightAndLevel { airspeed: 100.0 },
    //                 solver: None,
    //             },
    //         ))
    //         .id();

    //     // Run system and verify it handles extreme conditions gracefully
    //     app.update();

    //     let needs_trim = app
    //         .world()
    //         .get::<NeedsTrim>(aircraft_entity)
    //         .expect("NeedsTrim component should exist");

    //     assert!(
    //         needs_trim.solver.is_some(),
    //         "Solver should initialize even with invalid conditions"
    //     );
    // }
}
