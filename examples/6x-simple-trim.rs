use bevy::prelude::*;
use flyer::{
    components::{FullAircraftConfig, SimpleTrimSolver, TrimCondition, TrimState},
    resources::PhysicsConfig,
};
use std::fs::File;
use std::io::Write;

fn main() {
    // Create configuration for the trim solver
    let physics_config = PhysicsConfig::default();
    let aircraft_config = FullAircraftConfig::default();
    
    // Test straight and level flight at 70 m/s (upper cruise speed range)
    let condition = TrimCondition::StraightAndLevel { airspeed: 70.0 };
    
    // Create our simple trim solver with more relaxed tolerances for better convergence
    let mut trim_solver = SimpleTrimSolver::new(
        aircraft_config.clone(),
        condition,
        physics_config.clone(),
        10000,
        0.01,   // relaxed force tolerance (10x original)
        0.01,   // relaxed moment tolerance (10x original)
        true,
    );
    
    // Find the trim solution with multiple initial alpha values
    println!("Finding trim for straight and level flight at 70 m/s with wider alpha range...");
    
    // Try multiple initial guesses with different alpha values
    // Include our analytical solution (-0.043 radians or -2.46 degrees) as one of the guesses
    let alpha_values = [-0.2, -0.15, -0.1, -0.05, -0.043, 0.0, 0.05, 0.1, 0.15];
    let mut best_result = None;
    let mut best_cost = f64::MAX;
    
    for alpha in alpha_values.iter() {
        println!("\n--- Trying with initial alpha: {:.2}° ---", alpha * 180.0 / std::f64::consts::PI);
        
        // Create initial guess with this alpha
        let mut initial_guess = TrimState::default();
        initial_guess.longitudinal.alpha = *alpha;
        initial_guess.longitudinal.theta = *alpha; // For level flight, theta = alpha
        
        // Find trim with this initial guess
        let result = trim_solver.find_trim(Some(initial_guess));
        
        // Check if this is better than our previous best
        if result.cost < best_cost {
            best_cost = result.cost;
            best_result = Some(result);
            println!("  New best solution found!");
        }
    }
    
    // Unwrap the best result
    let result = best_result.unwrap();
    
    // Print the results
    println!("\n==== TRIM RESULT SUMMARY ====");
    println!("Converged: {}", result.converged);
    println!("Iterations: {}", result.iterations);
    println!("Final cost: {:.6}", result.cost);
    println!("\nFinal trim state:");
    println!("  Alpha: {:.2}°", result.state.longitudinal.alpha.to_degrees());
    println!("  Theta: {:.2}°", result.state.longitudinal.theta.to_degrees());
    println!("  Elevator: {:.3}", result.state.longitudinal.elevator);
    println!("  Throttle: {:.3}", result.state.longitudinal.power_lever);
    
    println!("\nResiduals:");
    println!("  Vertical force: {:.6}", result.residuals.longitudinal.vertical_force);
    println!("  Horizontal force: {:.6}", result.residuals.longitudinal.horizontal_force);
    println!("  Pitch moment: {:.6}", result.residuals.longitudinal.pitch_moment);
    println!("  Flight path error: {:.2}°", result.residuals.longitudinal.gamma_error.to_degrees());
    
    // Save results to file for verification
    let mut file = File::create("trim_verification.csv").unwrap();
    writeln!(
        file,
        "parameter,value\nalpha,{}\ntheta,{}\nelevator,{}\nthrottle,{}\ncost,{}\nvertical_force,{}\nhorizontal_force,{}\npitch_moment,{}\ngamma_error,{}",
        result.state.longitudinal.alpha,
        result.state.longitudinal.theta,
        result.state.longitudinal.elevator,
        result.state.longitudinal.power_lever,
        result.cost,
        result.residuals.longitudinal.vertical_force,
        result.residuals.longitudinal.horizontal_force,
        result.residuals.longitudinal.pitch_moment,
        result.residuals.longitudinal.gamma_error
    ).unwrap();
    
    println!("\nTrim results also saved to trim_verification.csv");
}