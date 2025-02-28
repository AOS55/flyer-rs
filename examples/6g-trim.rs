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
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(PhysicsPlugin::with_config(PhysicsConfig::default()))
        .add_plugins(EnvironmentPlugin::new());

    // Probe a specific point in detail first
    let test_point = (0.0f64, 0.5f64, 0.0f64); // (alpha, throttle, elevator)
    println!("Analyzing test point:");
    println!("  Alpha: {:.3} rad", test_point.0);
    println!("  Throttle: {:.3}", test_point.1);
    println!("  Elevator: {:.3}", test_point.2);

    let mut virtual_physics = VirtualPhysics::new(&PhysicsConfig::default());
    let (forces, moments, states) = simulate_trim_point(
        &mut virtual_physics,
        test_point.0,
        test_point.1,
        test_point.2,
    );

    println!("\nSimulation results:");
    for i in 0..states.len() {
        println!("\nStep {}:", i);
        println!("  Forces: {:?}", forces[i]);
        println!("  Moments: {:?}", moments[i]);
        println!("  Velocity: {:?}", states[i].0.velocity);
        println!(
            "  Flight path angle: {:.3}°",
            (-states[i].0.velocity.z / states[i].0.velocity.x)
                .atan()
                .to_degrees()
        );
    }

    // Now create the full map
    let mut file = File::create("trim_debug.csv").unwrap();
    writeln!(
        file,
        "alpha,elevator,throttle,cost,force_cost,moment_cost,gamma_cost,mu_cost,\
         force_x,force_y,force_z,moment_x,moment_y,moment_z,\
         velocity_x,velocity_y,velocity_z,flight_path_angle,velocity_variation"
    )
    .unwrap();

    for &alpha in &[-0.1f64, 0.0, 0.1] {
        for elevator in (0..20).map(|i| -0.05 + (i as f64) * 0.005) {
            for throttle in (0..20).map(|i| 0.3 + (i as f64) * 0.035) {
                let (forces, moments, states) =
                    simulate_trim_point(&mut virtual_physics, alpha, throttle, elevator);

                // Calculate mean forces/moments over the trajectory
                let mean_forces = forces.iter().sum::<Vector3<f64>>() / forces.len() as f64;
                let mean_moments = moments.iter().sum::<Vector3<f64>>() / moments.len() as f64;

                // Calculate state variations to capture dynamic behavior
                let velocity_variation = states
                    .windows(2)
                    .map(|w| (w[1].0.velocity - w[0].0.velocity).norm())
                    .sum::<f64>()
                    / (states.len() - 1) as f64;

                // Get final state for reference
                let final_state = states.last().unwrap();

                // Calculate cost components using means instead of final values
                let aircraft_config = FullAircraftConfig::default();
                let mass = aircraft_config.mass.mass;
                let gravity = 9.81;
                let wingspan = aircraft_config.geometry.wing_span;

                let characteristic_force = mass * gravity;
                let characteristic_moment = mass * gravity * wingspan;

                let normalized_forces = mean_forces / characteristic_force;
                let normalized_moments = mean_moments / characteristic_moment;

                let flight_path = (-final_state.0.velocity.z / final_state.0.velocity.x).atan();

                let force_cost = normalized_forces.norm_squared();
                let moment_cost = normalized_moments.norm_squared();
                let gamma_cost = flight_path.powi(2);
                let mu_cost = 0.0; // Straight & level flight

                // Use solver weights
                let total_cost =
                    1.0 * force_cost + 1.0 * moment_cost + 10.0 * gamma_cost + 10.0 * mu_cost;

                writeln!(
                    file,
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    alpha,
                    elevator,
                    throttle,
                    total_cost,
                    force_cost,
                    moment_cost,
                    gamma_cost,
                    mu_cost,
                    mean_forces.x,
                    mean_forces.y,
                    mean_forces.z,
                    mean_moments.x,
                    mean_moments.y,
                    mean_moments.z,
                    final_state.0.velocity.x,
                    final_state.0.velocity.y,
                    final_state.0.velocity.z,
                    flight_path.to_degrees(),
                    velocity_variation
                )
                .unwrap();
            }
        }
    }

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

    // Test points around expected trim condition
    let test_matrices = vec![
        // (alpha, throttle, elevator)
        (0.05, 0.5, 0.0),  // Baseline
        (0.05, 0.5, -0.1), // Elevator variation
        (0.05, 0.6, 0.0),  // Throttle variation
        (0.07, 0.5, 0.0),  // Alpha variation
    ];

    println!("\n=== Detailed Analysis of Test Points ===\n");

    for &(alpha, throttle, elevator) in &test_matrices {
        analyze_trim_point(
            &mut virtual_physics,
            alpha,
            throttle,
            elevator,
            &solver_config,
        );
    }

    // Create detailed cost landscape...
    create_cost_landscape(&mut virtual_physics, &solver_config);
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
