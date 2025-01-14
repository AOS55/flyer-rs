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

/// Modified trim solver using virtual physics
pub struct TrimSolver {
    pub virtual_physics: VirtualPhysics,
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
        initial_spatial_state: SpatialComponent,
        initial_prop_state: PropulsionState,
        aircraft_config: FullAircraftConfig,
        condition: TrimCondition,
        physics_config: &PhysicsConfig,
    ) -> Self {
        let mut virtual_physics = VirtualPhysics::new(physics_config);

        // Create initial state
        let virtual_aircraft = virtual_physics.spawn_aircraft(
            &initial_spatial_state,
            &initial_prop_state,
            &aircraft_config,
        );

        Self {
            virtual_physics,
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
        println!("Calculated residuals: {:?}", residuals);

        let cost = self.calculate_cost(&residuals);
        println!("Calculated cost: {}", cost);

        // 2. Update best solution if current is better
        println!("Comparing cost {} with best_cost {}", cost, self.best_cost);
        if cost < self.best_cost {
            println!(
                "Found better solution, updating best cost from {} to {}",
                self.best_cost, cost
            );
            self.best_cost = cost;
            self.best_state = self.current_state;
        }

        // 3. Update state using chosen optimization method
        if self.settings.use_gradient_refinement {
            println!("Using gradient optimization");
            self.gradient_optimization_step();
        } else {
            println!("Using pattern search");
            self.pattern_search_step();
        }

        println!("After optimization step:");
        println!("  Current state: {:?}", self.current_state);
        println!("  Best state: {:?}", self.best_state);
        println!("  Best cost: {}", self.best_cost);

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
        // Set control surfaces
        let control_surfaces = AircraftControlSurfaces {
            elevator: self.current_state.elevator,
            aileron: self.current_state.aileron,
            rudder: self.current_state.rudder,
            power_lever: self.current_state.power_lever,
        };

        // Set attitude
        let attitude = UnitQuaternion::from_euler_angles(
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
        let velocity = Vector3::new(vx, vy, vz);

        // Update virtual aircraft state
        self.virtual_physics
            .set_state(self.virtual_aircraft, &velocity, &attitude);
        self.virtual_physics
            .set_controls(self.virtual_aircraft, &control_surfaces);

        // Run virtual physics for a few steps to stabilize
        self.virtual_physics.run_steps(self.virtual_aircraft, 20);

        // Calculate forces at final state
        let (forces, moments) = self.virtual_physics.calculate_forces(self.virtual_aircraft);
        let (final_spatial, _final_controls) =
            self.virtual_physics.get_state(self.virtual_aircraft);

        TrimResiduals {
            forces,
            moments,
            gamma_error: match self.condition {
                TrimCondition::SteadyClimb { gamma, .. } => {
                    let actual_gamma =
                        (-final_spatial.velocity.z / final_spatial.velocity.x).atan();
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
        let moment_weight = 1.0;
        let gamma_weight = 0.1;
        let mu_weight = 0.1;

        // Calculate individual costs
        let force_cost = residuals.forces.norm_squared();
        let moment_cost = residuals.moments.norm_squared();
        let gamma_cost = residuals.gamma_error.powi(2);
        let mu_cost = residuals.mu_error.powi(2);

        println!("Cost calculation:");
        println!("  Force cost: {}", force_cost);
        println!("  Moment cost: {}", moment_cost);
        println!("  Gamma cost: {}", gamma_cost);
        println!("  Mu cost: {}", mu_cost);

        // Combine weighted costs
        let total_cost = force_weight * force_cost
            + moment_weight * moment_cost
            + gamma_weight * gamma_cost
            + mu_weight * mu_cost;

        println!("  Total cost: {}", total_cost);
        total_cost
    }

    fn gradient_optimization_step(&mut self) {
        // Calculate gradients using finite differences
        let gradients = self.calculate_gradients();

        // Simple gradient descent with line search
        let learning_rates = [0.01, 0.001, 0.0001];

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
        let deltas = [0.01, 0.001, 0.0001];

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
            max_iterations: 5,
            cost_tolerance: 1e-3,
            state_tolerance: 1e-4,
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
    fn test_solver_iteration() {
        let mut app = create_test_app();

        // Modify solver settings to make convergence slower
        app.world_mut()
            .resource_mut::<TrimSolverConfig>()
            .cost_tolerance = 1e-8; // Make convergence harder
        app.world_mut()
            .resource_mut::<TrimSolverConfig>()
            .max_iterations = 1000; // Allow more iterations

        let aircraft_entity = spawn_test_aircraft(&mut app);

        // Initial update to initialize
        println!("\n=== Initial Update ===");
        app.update();

        // Get initial cost
        let initial_cost = {
            let needs_trim = app
                .world()
                .get::<NeedsTrim>(aircraft_entity)
                .expect("NeedsTrim component should exist");

            let solver = needs_trim.solver.as_ref().unwrap();
            println!("\nAfter initialization:");
            println!("  Current state: {:?}", solver.current_state);
            println!("  Best cost: {}", solver.best_cost);

            needs_trim.solver.as_ref().unwrap().best_cost
        };

        // Track costs across iterations
        let mut costs = vec![initial_cost];

        // Run iterations and collect costs
        for i in 0..10 {
            app.update();
            println!("\n=== Running Iterations ===");
            if let Some(needs_trim) = app.world().get::<NeedsTrim>(aircraft_entity) {
                if let Some(ref solver) = needs_trim.solver {
                    println!("  Current cost: {}", solver.best_cost);
                    println!("  Iteration count: {}", solver.iteration);

                    costs.push(solver.best_cost);
                }
            } else {
                println!("NeedsTrim component removed after {} iterations", i + 1);
                break;
            }
        }

        // Find the minimum cost achieved
        let min_cost = costs.iter().copied().fold(f64::INFINITY, f64::min);

        assert!(
            min_cost < initial_cost,
            "Cost should decrease with iterations. Initial: {}, Current: {}",
            initial_cost,
            min_cost
        );
    }

    #[test]
    fn test_solver_convergence() {
        let mut app = create_test_app();
        let aircraft_entity = spawn_test_aircraft(&mut app);

        app.add_systems(Update, trim_aircraft_system);

        // Run until convergence or max iterations
        let max_updates = 20;
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
        let bounds = &solver.settings.bounds;

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
    fn test_solver_cost_calculation() {
        let mut app = create_test_app();
        let aircraft_entity = spawn_test_aircraft(&mut app);

        app.update();

        let needs_trim = app
            .world()
            .get::<NeedsTrim>(aircraft_entity)
            .expect("NeedsTrim component should exist");
        let solver = needs_trim.solver.as_ref().unwrap();

        // Create test residuals
        let perfect_residuals = TrimResiduals {
            forces: Vector3::zeros(),
            moments: Vector3::zeros(),
            gamma_error: 0.0,
            mu_error: 0.0,
        };

        let imperfect_residuals = TrimResiduals {
            forces: Vector3::new(100.0, 0.0, 100.0),
            moments: Vector3::new(10.0, 10.0, 10.0),
            gamma_error: 0.1,
            mu_error: 0.1,
        };

        // Test cost function behavior
        let perfect_cost = solver.calculate_cost(&perfect_residuals);

        assert!(perfect_cost >= 0.0, "Cost should never be negative");
        assert!(
            perfect_cost < solver.settings.cost_tolerance,
            "Perfect trim should have near-zero cost"
        );

        let imperfect_cost = solver.calculate_cost(&imperfect_residuals);

        assert!(
            imperfect_cost > 0.0,
            "Imperfect trim should have positive cost"
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
