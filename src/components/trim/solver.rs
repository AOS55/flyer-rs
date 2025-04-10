use argmin::{
    core::{CostFunction, Error, Executor, Gradient, IterState, OptimizationResult},
    solver::{linesearch::MoreThuenteLineSearch, quasinewton::LBFGS},
};
use bevy::prelude::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    components::{
        AircraftControlSurfaces, FullAircraftConfig, LongitudinalTrimState, PropulsionState,
        SpatialComponent, TrimCondition, TrimSolverConfig, TrimState,
    },
    resources::PhysicsConfig,
    systems::VirtualPhysics,
};

/// Simplified trim optimizer that uses only forces and moments
#[derive(Debug, Clone)]
pub struct TrimOptimizer {
    physics_config: PhysicsConfig,
    initial_spatial: SpatialComponent,
    initial_prop: PropulsionState,
    aircraft_config: FullAircraftConfig,
    condition: TrimCondition,
    debug_level: usize,
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
        Self {
            physics_config,
            initial_spatial,
            initial_prop,
            aircraft_config,
            condition,
            debug_level: settings.debug_level,
        }
    }

    fn create_virtual_physics(&self) -> (VirtualPhysics, Entity) {
        let mut virtual_physics = VirtualPhysics::new(&self.physics_config);
        let entity = virtual_physics.spawn_aircraft(
            &self.initial_spatial,
            &self.initial_prop,
            &self.aircraft_config,
        );
        (virtual_physics, entity)
    }

    /// Calculate forces and moments for a given state
    fn calculate_forces_and_moments(
        &self,
        state: &LongitudinalTrimState,
    ) -> (Vector3<f64>, Vector3<f64>) {
        let (mut virtual_physics, virtual_aircraft) = self.create_virtual_physics();

        // Debug output if needed
        if self.debug_level > 1 {
            println!("\n=== Force Calculation ===");
            println!("  Alpha: {:.1}°", state.alpha.to_degrees());
            println!("  Theta: {:.1}°", state.theta.to_degrees());
            println!("  Elevator: {:.3}", state.elevator);
            println!("  Throttle: {:.3}", state.power_lever);
        }

        // Use absolute alpha value to match the air data system convention
        let abs_alpha = state.alpha.abs();
        
        // Calculate flight path angle relative to horizon
        let flight_path_angle = match self.condition {
            TrimCondition::StraightAndLevel { .. } => state.theta - abs_alpha,
            TrimCondition::SteadyClimb { gamma, .. } => gamma,
            TrimCondition::CoordinatedTurn { .. } => state.theta - abs_alpha,
        };

        // Calculate world-frame velocity based on flight path
        let airspeed = match self.condition {
            TrimCondition::StraightAndLevel { airspeed } => airspeed,
            TrimCondition::SteadyClimb { airspeed, .. } => airspeed,
            TrimCondition::CoordinatedTurn { airspeed, .. } => airspeed,
        };

        // Set velocity in world frame aligned with flight path
        let velocity = Vector3::new(
            airspeed * flight_path_angle.cos(),
            0.0,  // No lateral velocity for longitudinal trim
            -airspeed * flight_path_angle.sin() // Negative because z is down in world frame
        );
        
        if self.debug_level > 1 {
            println!("DEBUG: Virtual physics state - Alpha: {:.2}°, FPA: {:.2}°", 
                     state.alpha.to_degrees(), flight_path_angle.to_degrees());
            println!("DEBUG: Velocity vector: [{:.1}, {:.1}, {:.1}]", 
                     velocity.x, velocity.y, velocity.z);
        }
        
        // Use theta directly from the optimizer
        let pitch = state.theta;
        
        let attitude = UnitQuaternion::from_euler_angles(0.0, pitch, 0.0);

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

        virtual_physics.calculate_forces(virtual_aircraft)
    }
}

// Implement CostFunction for TrimOptimizer
impl CostFunction for TrimOptimizer {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        // Convert parameters to trim state (with clamping to ensure reasonable values)
        let mut clamped_param = param.clone();
        
        // Clamp parameters to reasonable physical ranges
        // elevator: -1.0 to 1.0
        clamped_param[0] = clamped_param[0].clamp(-1.0, 1.0);
        // power_lever: 0.0 to 1.0
        clamped_param[1] = clamped_param[1].clamp(0.0, 1.0);
        // alpha: -0.17 to 0.35 rad (-10 to 20 degrees) - widened to accommodate analytical solution
        clamped_param[2] = clamped_param[2].clamp(-0.17, 0.35);
        
        // For level flight, enforce theta = alpha (critical for physical realism)
        // This reduces the optimization dimensions and enforces correct level flight physics
        if let TrimCondition::StraightAndLevel { .. } = self.condition {
            clamped_param[3] = clamped_param[2]; // Make theta = alpha
        } else {
            // For other conditions, just clamp theta to reasonable values
            clamped_param[3] = clamped_param[3].clamp(-0.17, 0.35);
        }
        
        let state = LongitudinalTrimState::from_vector(&clamped_param);
        
        // Calculate forces and moments
        let (forces, moments) = self.calculate_forces_and_moments(&state);
        
        // Apply normalization to prevent numerical issues
        let cost = forces.norm_squared() + moments.norm_squared();
        
        // Enhanced debug output for tracing issues
        if self.debug_level > 0 {
            // Show the original and clamped parameter values to understand clamping effects
            if param != &clamped_param {
                println!(
                    "CLAMPING: Original: [elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°] → Clamped: [elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°]",
                    param[0], param[1], param[2].to_degrees(), param[3].to_degrees(),
                    clamped_param[0], clamped_param[1], clamped_param[2].to_degrees(), clamped_param[3].to_degrees()
                );
            }
            
            println!(
                "Cost: {:.6} | Params: [elev={:.3}, pwr={:.3}, α={:.1}°, θ={:.1}°] | Forces: [{:.1}, {:.1}, {:.1}] N | Moments: [{:.1}, {:.1}, {:.1}] Nm",
                cost,
                state.elevator,
                state.power_lever,
                state.alpha.to_degrees(),
                state.theta.to_degrees(),
                forces.x, forces.y, forces.z,
                moments.x, moments.y, moments.z
            );
        }
        
        // Extra diagnostics when debug level is higher
        if self.debug_level > 1 {
            // Create a test case with standard inputs to verify the force model
            let test_state = LongitudinalTrimState {
                elevator: 0.0,
                power_lever: 0.5,
                alpha: 0.05,  // ~3 degrees
                theta: 0.05,
            };
            
            let (test_forces, test_moments) = self.calculate_forces_and_moments(&test_state);
            println!("DIAGNOSTIC CASE: α=3.0°, elev=0.0, pwr=0.5 → F=[{:.1}, {:.1}, {:.1}], M=[{:.1}, {:.1}, {:.1}]", 
                     test_forces.x, test_forces.y, test_forces.z,
                     test_moments.x, test_moments.y, test_moments.z);
                     
            // Get detailed info about the aircraft model being used
            println!("AIRCRAFT MODEL: {}", self.aircraft_config.name);
            println!("  Wing Area: {:.1} m²", self.aircraft_config.geometry.wing_area);
            println!("  Wing Span: {:.1} m", self.aircraft_config.geometry.wing_span);
            println!("  MAC: {:.2} m", self.aircraft_config.geometry.mac);
            
            // Print key aerodynamic coefficients to verify they match what we expect
            let aero = &self.aircraft_config.aero_coef;
            println!("AERO COEFFICIENTS:");
            println!("  c_l_0: {:.3}, c_l_alpha: {:.3}, c_l_deltae: {:.3}", 
                    aero.lift.c_l_0, aero.lift.c_l_alpha, aero.lift.c_l_deltae);
            println!("  c_m_0: {:.3}, c_m_alpha: {:.3}, c_m_deltae: {:.3}", 
                    aero.pitch.c_m_0, aero.pitch.c_m_alpha, aero.pitch.c_m_deltae);
            
            // Make it easy to see that these are the values from the optimizer's internal model
            println!("NOTE: Above forces/moments are from the OPTIMIZER'S INTERNAL MODEL, not the simulation");
        }
        
        Ok(cost)
    }
}

// Implement Gradient for TrimOptimizer using finite differences
impl Gradient for TrimOptimizer {
    type Param = Vec<f64>;
    type Gradient = Vec<f64>;

    fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
        // Adaptive finite difference gradient calculation
        let base_step = 1e-3; // Larger base step size
        let mut grad = vec![0.0; param.len()];
        
        // Apply clamping to base parameters first to ensure we start from valid point
        let mut clamped_param = param.clone();
        // elevator: -1.0 to 1.0
        clamped_param[0] = clamped_param[0].clamp(-1.0, 1.0);
        // power_lever: 0.0 to 1.0
        clamped_param[1] = clamped_param[1].clamp(0.0, 1.0);
        // alpha: -0.17 to 0.35 rad (-10 to 20 degrees)
        clamped_param[2] = clamped_param[2].clamp(-0.17, 0.35);
        
        // For level flight, enforce theta = alpha (critical for physical realism)
        if let TrimCondition::StraightAndLevel { .. } = self.condition {
            clamped_param[3] = clamped_param[2]; // Make theta = alpha
        } else {
            // For other conditions, just clamp theta to reasonable values
            clamped_param[3] = clamped_param[3].clamp(-0.17, 0.35);
        }
        
        let base_cost = self.cost(&clamped_param)?;
        
        for i in 0..param.len() {
            // Adaptive step size based on parameter magnitude
            let h = base_step * (1.0 + clamped_param[i].abs());
            
            // Forward difference with clamping
            let mut param_plus_h = clamped_param.clone();
            param_plus_h[i] += h;
            
            // Re-apply clamping to ensure we're still in valid parameter space
            if i == 0 {
                param_plus_h[i] = param_plus_h[i].clamp(-1.0, 1.0); // elevator
            } else if i == 1 {
                param_plus_h[i] = param_plus_h[i].clamp(0.0, 1.0);  // power_lever
            } else if i == 2 || i == 3 {
                param_plus_h[i] = param_plus_h[i].clamp(-0.17, 0.35); // alpha or theta
            }
            
            // For level flight special case with theta = alpha
            if i == 2 {
                if let TrimCondition::StraightAndLevel { .. } = self.condition {
                    param_plus_h[3] = param_plus_h[2]; // Update theta when alpha changes
                }
            }
            
            let cost_plus_h = self.cost(&param_plus_h)?;
            grad[i] = (cost_plus_h - base_cost) / h;
        }
        
        Ok(grad)
    }
}

/// Result of a trim operation
#[derive(Debug, Clone)]
pub struct TrimResult {
    pub state: TrimState,
    pub cost: f64,
    pub iterations: usize,
    pub converged: bool,
}

/// Simplified trim solver that uses only LBFGS
pub struct TrimSolver {
    optimizer: TrimOptimizer,
    current_state: TrimState,
    best_state: TrimState,
    best_cost: f64,
    iteration: usize,
    settings: TrimSolverConfig,
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
            settings,
        }
    }

    pub fn initialize(&mut self, initial_guess: TrimState) {
        self.current_state = initial_guess;
        self.best_state = initial_guess;
        self.best_cost = f64::MAX;
        self.iteration = 0;
    }
    
    /// Get the current trim state
    pub fn current_state(&self) -> TrimState {
        self.current_state.clone()
    }

    pub fn iterate(&mut self) -> Result<bool, Error> {
        // Convert current state to parameter vector
        let param = self.current_state.longitudinal.to_vector();
        
        // Perform a single LBFGS step
        let result = self.gradient_step(&param)?;
        
        // Update state if we have a result
        if let Some(param) = &result.state.param {
            let current_cost = result.state.cost;
            
            // Convert parameters back to trim state
            let long_state = LongitudinalTrimState::from_vector(param);
            let mut new_state = self.current_state;
            new_state.longitudinal = long_state;
            
            // Track best solution
            if current_cost < self.best_cost {
                self.best_cost = current_cost;
                self.best_state = new_state;
            }
            
            self.current_state = new_state;
        }
        
        self.iteration += 1;
        Ok(self.has_converged())
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
        // Configure line search
        let linesearch = MoreThuenteLineSearch::new()
            .with_c(1e-4, 0.9)
            .expect("Failed to configure line search");
        
        // Create LBFGS solver
        let solver = LBFGS::new(linesearch, 5);
        
        // Run a single iteration
        Executor::new(self.optimizer.clone(), solver)
            .configure(|state| {
                state
                    .param(init_param.to_vec())
                    .max_iters(1)
                    .target_cost(self.settings.cost_tolerance)
            })
            .run()
    }
    
    pub fn has_converged(&self) -> bool {
        // Check if cost is below tolerance or max iterations reached
        self.best_cost < self.settings.cost_tolerance || self.iteration >= self.settings.max_iterations
    }
    
    pub fn get_best_solution(&self) -> TrimResult {
        TrimResult {
            state: self.best_state.clone(),
            cost: self.best_cost,
            iterations: self.iteration,
            converged: self.best_cost < self.settings.cost_tolerance,
        }
    }
}
