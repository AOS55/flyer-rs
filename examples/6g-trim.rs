use bevy::prelude::*;
use flyer::{
    components::{
        AircraftControlSurfaces, FullAircraftConfig, LongitudinalBounds, LongitudinalResiduals,
        PropulsionState, SpatialComponent, TrimCondition, TrimSolverConfig,
    },
    plugins::{EnvironmentPlugin, PhysicsPlugin},
    resources::PhysicsConfig,
    systems::VirtualPhysics,
};
use nalgebra::{UnitQuaternion, Vector3};
use std::{fs::File, io::Write};

fn main() {
    println!("Testing trim convergence for straight and level flight");
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(PhysicsPlugin::with_config(PhysicsConfig::default()))
        .add_plugins(EnvironmentPlugin::new());

    let mut virtual_physics = VirtualPhysics::new(&PhysicsConfig::default());
    
    // Configure trim solver with optimized settings
    let solver_config = TrimSolverConfig {
        max_iterations: 100,
        cost_tolerance: 1e-3,
        use_gradient_refinement: true,
        longitudinal_bounds: LongitudinalBounds {
            elevator_range: (-0.5, 0.5),
            throttle_range: (0.1, 0.9),
            alpha_range: (-0.2, 0.2),
            theta_range: (-0.2, 0.2),
        },
        ..Default::default()
    };

    // Define a range of test points we'll evaluate for trim quality
    // We'll focus on finding a good trim solution for straight and level flight
    println!("\n=== Testing Trim Solutions for Straight & Level Flight ===");

    // Test various trim configurations
    let trim_configs = vec![
        // Initial exploratory tests
        (-0.02, 0.60, -0.05),  // Slightly nose down, moderate power, elevator up
        (-0.01, 0.55, -0.03),  // Near level, moderate power, slight elevator up
        (0.00, 0.50, 0.00),    // Level, moderate power, neutral elevator
        (0.01, 0.45, 0.02),    // Slightly nose up, lower power, slight elevator down
        
        // Previous tests suggested around here
        (0.03, 0.51, -0.01),   // Small positive alpha, moderate power, slight elevator up
        (-0.01, 0.56, -0.02),  // Small negative alpha, moderate power, slight elevator up
        
        // Based on optimizer outputs
        (-0.08, 0.75, -0.05),  // Negative alpha, higher power, elevator up
        (-0.10, 0.80, -0.06),  // More negative alpha, higher power, elevator up
    ];

    // Store the best configurations
    let mut best_config = trim_configs[0];
    let mut lowest_cost = f64::INFINITY;
    let mut best_results = None;

    println!("\nEvaluating potential trim solutions:");
    println!("-------------------------------------");
    
    for (i, &(alpha, throttle, elevator)) in trim_configs.iter().enumerate() {
        println!("\nTest Configuration #{}: ", i+1);
        println!("  Alpha: {:.2}° ({:.3} rad)", (alpha as f64).to_degrees(), alpha);
        println!("  Throttle: {:.3}", throttle);
        println!("  Elevator: {:.3}", elevator);
        
        // Analyze the trim point in detail
        let results = analyze_trim_point_detailed(
            &mut virtual_physics,
            alpha,
            throttle,
            elevator,
            &solver_config,
        );
        
        println!("  Trim cost: {:.6}", results.cost);
        
        // Track the best trim solution
        if results.cost < lowest_cost {
            lowest_cost = results.cost;
            best_config = (alpha, throttle, elevator);
            best_results = Some(results);
        }
    }
    
    // Analyze the best configuration in detail
    println!("\n\n=== BEST TRIM SOLUTION ===");
    println!("Alpha: {:.2}° ({:.3} rad)", (best_config.0 as f64).to_degrees(), best_config.0);
    println!("Throttle: {:.3}", best_config.1);
    println!("Elevator: {:.3}", best_config.2);
    println!("Cost: {:.6}", lowest_cost);
    
    let best_result = best_results.unwrap();
    
    println!("\nForce Balance:");
    println!("  Vertical force: {:.6} (target: 0)", best_result.residuals.vertical_force);
    println!("  Horizontal force: {:.6} (target: 0)", best_result.residuals.horizontal_force);
    println!("  Pitch moment: {:.6} (target: 0)", best_result.residuals.pitch_moment);
    println!("  Flight path angle: {:.3}° (target: 0°)", best_result.residuals.gamma_error.to_degrees());
    
    // Extended stability simulation for the best config
    println!("\nRunning extended stability simulation...");
    let (forces, moments, states) = simulate_trim_point(
        &mut virtual_physics,
        best_config.0,
        best_config.1,
        best_config.2
    );
    
    // Calculate stability metrics
    let velocity_variations: Vec<f64> = states
        .windows(2)
        .map(|w| (w[1].0.velocity - w[0].0.velocity).norm())
        .collect();
    
    let mean_variation = velocity_variations.iter().sum::<f64>() / velocity_variations.len() as f64;
    let max_variation = velocity_variations.iter().fold(0.0f64, |a, &b| a.max(b));
    
    println!("Stability metrics:");
    println!("  Mean velocity variation: {:.6}", mean_variation);
    println!("  Max velocity variation: {:.6}", max_variation);
    
    // Calculate final mean forces and moments
    let mean_forces = forces.iter().sum::<Vector3<f64>>() / forces.len() as f64;
    let mean_moments = moments.iter().sum::<Vector3<f64>>() / moments.len() as f64;
    
    // Get final state
    let final_state = states.last().unwrap();
    let (roll, pitch, yaw) = final_state.0.attitude.euler_angles();
    let flight_path = (-final_state.0.velocity.z / final_state.0.velocity.x).atan();
    
    println!("\nFinal state after extended simulation:");
    println!("  Roll: {:.3}°", roll.to_degrees());
    println!("  Pitch: {:.3}°", pitch.to_degrees());
    println!("  Yaw: {:.3}°", yaw.to_degrees());
    println!("  Flight path: {:.3}°", flight_path.to_degrees());
    println!("  Airspeed: {:.1} m/s", final_state.0.velocity.norm());
    
    // Verify the trim solution meets our criteria
    let aircraft_config = FullAircraftConfig::default();
    let characteristic_force = aircraft_config.mass.mass * 9.81;
    let characteristic_moment = characteristic_force * aircraft_config.geometry.wing_span;
    
    // Normalize for assertions
    let norm_vf = mean_forces.z / characteristic_force;
    let norm_hf = mean_forces.x / characteristic_force;
    let norm_pm = mean_moments.y / characteristic_moment;
    
    println!("\nVerification of trim constraints:");
    
    // Vertical force (lift balances weight)
    let vf_ok = norm_vf.abs() < 0.1;
    println!("  ✓ Vertical force balance: {:.4} ({}) - should be near 0.0", 
             norm_vf, if vf_ok { "PASS" } else { "FAIL" });
    
    // Horizontal force (thrust balances drag)
    let hf_ok = norm_hf.abs() < 0.2;  // More tolerance for horizontal equilibrium
    println!("  ✓ Horizontal force balance: {:.4} ({}) - should be near 0.0", 
             norm_hf, if hf_ok { "PASS" } else { "FAIL" });
    
    // Pitch moment (nose-up/down torque)
    let pm_ok = norm_pm.abs() < 0.01;
    println!("  ✓ Pitch moment balance: {:.4} ({}) - should be near 0.0", 
             norm_pm, if pm_ok { "PASS" } else { "FAIL" });
    
    // Flight path (straight)
    let fp_ok = flight_path.abs().to_degrees() < 2.0;
    println!("  ✓ Flight path angle: {:.3}° ({}) - should be near 0.0°", 
             flight_path.to_degrees(), if fp_ok { "PASS" } else { "FAIL" });
    
    // Roll angle (level)
    let roll_ok = roll.abs().to_degrees() < 1.0;
    println!("  ✓ Roll angle: {:.3}° ({}) - should be near 0.0°", 
             roll.to_degrees(), if roll_ok { "PASS" } else { "FAIL" });
    
    // Overall assessment
    let trim_ok = vf_ok && hf_ok && pm_ok && fp_ok && roll_ok;
    println!("\nTrim solution is {}!", if trim_ok { "VALID" } else { "INVALID" });
    
    // Save results to file for visualization
    let mut file = File::create("trim_verification.csv").unwrap();
    writeln!(
        file,
        "step,time,force_x,force_y,force_z,moment_x,moment_y,moment_z,vx,vy,vz,roll,pitch,yaw"
    ).unwrap();
    
    for (i, ((force, moment), state)) in forces.iter().zip(moments.iter()).zip(states.iter()).enumerate() {
        let (roll, pitch, yaw) = state.0.attitude.euler_angles();
        writeln!(
            file,
            "{},{:.3},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            i,
            i as f64 * 0.01, // Assuming dt=0.01s
            force.x,
            force.y,
            force.z,
            moment.x,
            moment.y,
            moment.z,
            state.0.velocity.x,
            state.0.velocity.y,
            state.0.velocity.z,
            roll.to_degrees(),
            pitch.to_degrees(),
            yaw.to_degrees()
        ).unwrap();
    }
    
    println!("\nDetailed results saved to trim_verification.csv");
}

fn simulate_trim_point(
    virtual_physics: &mut VirtualPhysics,
    alpha: f64,
    throttle: f64,
    elevator: f64,
) -> (
    Vec<Vector3<f64>>,
    Vec<Vector3<f64>>,
    Vec<(SpatialComponent, AircraftControlSurfaces)>,
) {
    // For straight and level flight, desired flight path angle is zero
    let flight_path_angle: f64 = 0.0;

    // Pitch attitude is alpha for trim
    let theta = alpha;
    let attitude = UnitQuaternion::from_euler_angles(0.0, theta, 0.0);

    let airspeed = 100.0;
    // Velocity aligned with desired flight path
    let velocity = Vector3::new(
        airspeed * flight_path_angle.cos(),
        0.0,
        -airspeed * flight_path_angle.sin(),
    );

    let aircraft = virtual_physics.spawn_aircraft(
        &SpatialComponent {
            position: Vector3::new(0.0, 0.0, -1000.0),
            velocity,
            attitude,
            angular_velocity: Vector3::zeros(),
        },
        &PropulsionState::default(),
        &FullAircraftConfig::default(),
    );

    let controls = AircraftControlSurfaces {
        elevator,
        aileron: 0.0,
        rudder: 0.0,
        power_lever: throttle,
    };
    virtual_physics.set_controls(aircraft, &controls);

    // Allow simulation to settle
    let settling_steps = 200;
    virtual_physics.run_steps(aircraft, settling_steps);

    let mut forces = Vec::new();
    let mut moments = Vec::new();
    let mut states = Vec::new();

    // Collect data for analysis
    let num_steps = 50;
    for _ in 0..num_steps {
        virtual_physics.run_steps(aircraft, 1);
        let (force, moment) = virtual_physics.calculate_forces(aircraft);
        let (state, control) = virtual_physics.get_state(aircraft);
        forces.push(force);
        moments.push(moment);
        states.push((state, control));
    }

    (forces, moments, states)
}

fn analyze_trim_point(
    virtual_physics: &mut VirtualPhysics,
    alpha: f64,
    throttle: f64,
    elevator: f64,
    solver_config: &TrimSolverConfig,
) {
    println!("Testing configuration:");
    println!("  Alpha: {:.3}° ({:.3} rad)", alpha.to_degrees(), alpha);
    println!("  Throttle: {:.3}", throttle);
    println!("  Elevator: {:.3}", elevator);

    let (forces, moments, states) = simulate_trim_point(virtual_physics, alpha, throttle, elevator);

    // Calculate residuals
    let condition = TrimCondition::StraightAndLevel { airspeed: 100.0 };
    let aircraft_config = FullAircraftConfig::default();
    let characteristic_force = aircraft_config.mass.mass * 9.81;
    let characteristic_moment = characteristic_force * aircraft_config.geometry.wing_span;

    let mean_forces = forces.iter().sum::<Vector3<f64>>() / forces.len() as f64;
    let mean_moments = moments.iter().sum::<Vector3<f64>>() / moments.len() as f64;

    // Calculate residuals
    let residuals = LongitudinalResiduals {
        vertical_force: mean_forces.z / characteristic_force,
        horizontal_force: mean_forces.x / characteristic_force,
        pitch_moment: mean_moments.y / characteristic_moment,
        gamma_error: (-states.last().unwrap().0.velocity.z / states.last().unwrap().0.velocity.x)
            .atan(),
    };

    // Calculate constraint penalties
    let bounds = &solver_config.longitudinal_bounds;
    let elevator_penalty = calculate_constraint_penalty(elevator, bounds.elevator_range, 20.0);
    let throttle_penalty = calculate_constraint_penalty(throttle, bounds.throttle_range, 20.0);
    let alpha_penalty = calculate_constraint_penalty(alpha, bounds.alpha_range, 30.0);

    // Calculate costs with original and new weightings
    let original_cost = 10.0 * residuals.vertical_force.powi(2)
        + 5.0 * residuals.horizontal_force.powi(2)
        + 2.0 * residuals.pitch_moment.powi(2)
        + 10.0 * residuals.gamma_error.powi(2);

    let constrained_cost = original_cost + elevator_penalty + throttle_penalty + alpha_penalty;

    println!("\nResidual Analysis:");
    println!("  Vertical Force: {:.6}", residuals.vertical_force);
    println!("  Horizontal Force: {:.6}", residuals.horizontal_force);
    println!("  Pitch Moment: {:.6}", residuals.pitch_moment);
    println!("  Gamma Error: {:.6}°", residuals.gamma_error.to_degrees());

    println!("\nConstraint Penalties:");
    println!("  Elevator: {:.6}", elevator_penalty);
    println!("  Throttle: {:.6}", throttle_penalty);
    println!("  Alpha: {:.6}", alpha_penalty);

    println!("\nCost Analysis:");
    println!("  Original Cost: {:.6}", original_cost);
    println!("  Constrained Cost: {:.6}", constrained_cost);

    // Add stability analysis
    analyze_stability(&states);
}

fn calculate_constraint_penalty(value: f64, range: (f64, f64), weight: f64) -> f64 {
    let (min, max) = range;
    let below_min = if value < min {
        (min - value).powi(2)
    } else {
        0.0
    };
    let above_max = if value > max {
        (value - max).powi(2)
    } else {
        0.0
    };
    weight * (below_min + above_max)
}

fn analyze_stability(states: &[(SpatialComponent, AircraftControlSurfaces)]) {
    let velocity_variations: Vec<f64> = states
        .windows(2)
        .map(|w| (w[1].0.velocity - w[0].0.velocity).norm())
        .collect();

    let mean_variation = velocity_variations.iter().sum::<f64>() / velocity_variations.len() as f64;
    let max_variation = velocity_variations.iter().fold(0.0f64, |a, &b| a.max(b));

    println!("\nStability Analysis:");
    println!("  Mean Velocity Variation: {:.6}", mean_variation);
    println!("  Max Velocity Variation: {:.6}", max_variation);
}

// Define a result struct for analyze_trim_point_detailed
struct TrimAnalysisResult {
    cost: f64,
    residuals: LongitudinalResiduals,
}

fn analyze_trim_point_detailed(
    virtual_physics: &mut VirtualPhysics,
    alpha: f64,
    throttle: f64,
    elevator: f64,
    solver_config: &TrimSolverConfig,
) -> TrimAnalysisResult {
    // Simulate the trim point as in analyze_trim_point
    let (forces, moments, states) = simulate_trim_point(virtual_physics, alpha, throttle, elevator);

    // Calculate residuals
    let aircraft_config = FullAircraftConfig::default();
    let characteristic_force = aircraft_config.mass.mass * 9.81;
    let characteristic_moment = characteristic_force * aircraft_config.geometry.wing_span;

    let mean_forces = forces.iter().sum::<Vector3<f64>>() / forces.len() as f64;
    let mean_moments = moments.iter().sum::<Vector3<f64>>() / moments.len() as f64;

    // Calculate residuals
    let residuals = LongitudinalResiduals {
        vertical_force: mean_forces.z / characteristic_force,
        horizontal_force: mean_forces.x / characteristic_force,
        pitch_moment: mean_moments.y / characteristic_moment,
        gamma_error: (-states.last().unwrap().0.velocity.z / states.last().unwrap().0.velocity.x)
            .atan(),
    };

    // Calculate constraint penalties
    let bounds = &solver_config.longitudinal_bounds;
    let elevator_penalty = calculate_constraint_penalty(elevator, bounds.elevator_range, 20.0);
    let throttle_penalty = calculate_constraint_penalty(throttle, bounds.throttle_range, 20.0);
    let alpha_penalty = calculate_constraint_penalty(alpha, bounds.alpha_range, 30.0);

    // Calculate cost with original weightings
    let original_cost = 10.0 * residuals.vertical_force.powi(2)
        + 5.0 * residuals.horizontal_force.powi(2)
        + 2.0 * residuals.pitch_moment.powi(2)
        + 10.0 * residuals.gamma_error.powi(2);

    let constrained_cost = original_cost + elevator_penalty + throttle_penalty + alpha_penalty;

    TrimAnalysisResult {
        cost: constrained_cost,
        residuals,
    }
}

fn create_cost_landscape(virtual_physics: &mut VirtualPhysics, solver_config: &TrimSolverConfig) {
    let mut file = File::create("trim_cost_landscape.csv").unwrap();
    // Add CSV header...

    // Create a finer mesh around promising regions
    for &alpha in &[-0.05f64, 0.0, 0.05, 0.1] {
        for elevator in (0..30).map(|i| -0.3 + (i as f64) * 0.02) {
            for throttle in (0..30).map(|i| 0.3 + (i as f64) * 0.02) {
                // Simulate and analyze point...
                // Write results to CSV...
            }
        }
    }
}
