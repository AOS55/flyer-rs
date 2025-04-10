fn main() {}

// use bevy::prelude::*;
// use flyer::{
//     components::{
//         AirData, AircraftControlSurfaces, FullAircraftConfig, LateralBounds, LongitudinalBounds,
//         NeedsTrim, PropulsionState, SpatialComponent, TrimCondition, TrimRequest, TrimSolverConfig,
//     },
//     plugins::{EnvironmentPlugin, PhysicsPlugin, TransformationPlugin},
//     resources::PhysicsConfig,
//     systems::{
//         aero_force_system, air_data_system, force_calculator_system, handle_trim_requests,
//         physics_integrator_system, trim_aircraft_system,
//     },
// };
// use nalgebra::{UnitQuaternion, Vector3};

// // Define tracking resource for convergence monitoring
// #[derive(Resource)]
// struct TrimConvergenceTracker {
//     last_cost: f64,
//     iterations: u32,
//     trim_complete: bool,
// }

// impl Default for TrimConvergenceTracker {
//     fn default() -> Self {
//         Self {
//             last_cost: f64::INFINITY,
//             iterations: 0,
//             trim_complete: false,
//         }
//     }
// }

// fn main() {
//     let mut app = App::new();

//     // Add minimal plugins
//     app.add_plugins(MinimalPlugins);

//     // Use a fixed timestep (100Hz)
//     app.insert_resource(Time::<Fixed>::from_hz(100.0));
//     println!("Using fixed timestep of 1/100 second");

//     // Set up trim request event
//     app.add_event::<TrimRequest>();

//     // Configure physics
//     let physics_config = PhysicsConfig::default();
//     app.add_plugins((
//         PhysicsPlugin::with_config(physics_config.clone()),
//         TransformationPlugin::default(),
//         EnvironmentPlugin::new(),
//     ));

//     // Initialize tracking resource
//     app.insert_resource(TrimConvergenceTracker::default());

//     // Configure trim solver with appropriate settings
//     app.insert_resource(TrimSolverConfig {
//         max_iterations: 50,
//         cost_tolerance: 1e-3,
//         use_gradient_refinement: true,
//         lateral_bounds: LateralBounds::default(),
//         longitudinal_bounds: LongitudinalBounds {
//             elevator_range: (-0.5, 0.5),
//             throttle_range: (0.2, 0.9),
//             alpha_range: (-0.2, 0.2),
//             theta_range: (-0.3, 0.3),
//         },
//         debug_level: 1, // Print every 10 iterations
//     });

//     // Setup aircraft components
//     let spatial = SpatialComponent {
//         position: Vector3::new(0.0, 0.0, -1000.0), // 1000m altitude
//         velocity: Vector3::new(50.0, 0.0, 0.0),    // Initial velocity 50 m/s
//         attitude: UnitQuaternion::identity(),      // Level attitude
//         angular_velocity: Vector3::zeros(),        // No rotation
//     };

//     let controls = AircraftControlSurfaces {
//         elevator: 0.0,
//         aileron: 0.0,
//         rudder: 0.0,
//         power_lever: 0.5, // Start at 50% throttle
//     };

//     let air_data = AirData {
//         true_airspeed: 50.0,
//         alpha: 0.05,
//         beta: 0.0,
//         dynamic_pressure: 0.5 * 1.225 * 50.0 * 50.0,
//         density: 1.225,
//         relative_velocity: Vector3::new(50.0, 0.0, 0.0),
//         wind_velocity: Vector3::zeros(),
//     };

//     let propulsion = PropulsionState::default();

//     // Create aircraft configuration
//     let aircraft_config = FullAircraftConfig::default();

//     // Spawn the entity with all required components
//     app.world_mut().spawn((
//         spatial.clone(),
//         controls,
//         propulsion,
//         air_data,
//         aircraft_config.clone(),
//     ));

//     println!("Aircraft entity created successfully");

//     // Add systems in correct order
//     app.add_systems(
//         FixedUpdate,
//         (
//             // Physics systems
//             air_data_system,
//             aero_force_system,
//             force_calculator_system,
//             physics_integrator_system,
//             // Trim systems
//             request_trim.run_if(|tracker: Res<TrimConvergenceTracker>| tracker.iterations == 0),
//             handle_trim_requests,
//             trim_aircraft_system,
//             // Monitor system (runs last)
//             monitor_trim_convergence,
//         )
//             .chain(),
//     );

//     println!("Starting application loop");

//     // Run until trim is complete or max iterations reached
//     let mut max_iterations = 1000; // Safety limit
//     while max_iterations > 0 {
//         app.update();

//         // Check if trim is complete
//         if app
//             .world()
//             .resource::<TrimConvergenceTracker>()
//             .trim_complete
//         {
//             break;
//         }

//         max_iterations -= 1;
//     }

//     println!("Trim process ended");
// }

// // System to request trim for aircraft
// fn request_trim(
//     mut trim_requests: EventWriter<TrimRequest>,
//     query: Query<Entity, With<SpatialComponent>>,
// ) {
//     // Find the aircraft entity
//     if let Some(entity) = query.iter().next() {
//         println!("Requesting trim for entity {:?}", entity);

//         // Send straight and level trim request at 50 m/s
//         trim_requests.send(TrimRequest {
//             entity,
//             condition: TrimCondition::StraightAndLevel { airspeed: 50.0 },
//         });

//         println!("Trim request sent for straight and level flight at 50 m/s");
//     }
// }

// // System to monitor trim convergence
// fn monitor_trim_convergence(
//     query: Query<(
//         &SpatialComponent,
//         &AircraftControlSurfaces,
//         &AirData,
//         Option<&NeedsTrim>,
//     )>,
//     mut tracker: ResMut<TrimConvergenceTracker>,
// ) {
//     // Create a timer to print status periodically
//     static mut PRINT_COUNTER: usize = 0;
//     let should_print = unsafe {
//         PRINT_COUNTER += 1;
//         PRINT_COUNTER % 10 == 0 // Print every 10 iterations
//     };

//     for (spatial, controls, air_data, needs_trim) in query.iter() {
//         if let Some(needs_trim) = needs_trim {
//             if let Some(ref solver) = needs_trim.solver {
//                 let current_cost = solver.best_cost;
//                 tracker.iterations += 1;

//                 // Check for valid cost
//                 if !current_cost.is_finite() {
//                     println!(
//                         "Warning: Cost became non-finite at iteration {}",
//                         tracker.iterations
//                     );
//                     return;
//                 }

//                 // Print status if cost changed significantly or periodically
//                 if (tracker.last_cost - current_cost).abs() > 1e-2 || should_print {
//                     // Extract relevant state parameters
//                     let (_, pitch, _) = spatial.attitude.euler_angles();

//                     println!(
//                         "Iteration {}: Cost = {:.6}\n  Airspeed = {:.1} m/s, Alpha = {:.1}°, Beta = {:.1}°, Pitch = {:.1}°\n  Elevator = {:.3}, Aileron = {:.3}, Rudder = {:.3}, Throttle = {:.3}",
//                         tracker.iterations,
//                         current_cost,
//                         spatial.velocity.norm(),
//                         air_data.alpha.to_degrees(),
//                         air_data.beta.to_degrees(),
//                         pitch.to_degrees(),
//                         controls.elevator,
//                         controls.aileron,
//                         controls.rudder,
//                         controls.power_lever,
//                     );
//                 }

//                 tracker.last_cost = current_cost;

//                 // Check for convergence
//                 if solver.has_converged() {
//                     println!(
//                         "Solver reports convergence at iteration {} with cost {:.6}",
//                         tracker.iterations, current_cost
//                     );
//                 }
//             }
//         } else if tracker.iterations > 0 {
//             // Trim is complete
//             let (roll, pitch, yaw) = spatial.attitude.euler_angles();

//             println!("\n=== TRIM COMPLETE ===");
//             println!("\nAircraft State:");
//             println!("----------------");
//             println!("  Airspeed:      {:.2} m/s", spatial.velocity.norm());
//             println!("  Altitude:      {:.2} m", -spatial.position.z);
//             println!("  Position:      [{:.2}, {:.2}, {:.2}]",
//                 spatial.position.x, spatial.position.y, spatial.position.z);
//             println!("  Velocity:      [{:.2}, {:.2}, {:.2}] m/s",
//                 spatial.velocity.x, spatial.velocity.y, spatial.velocity.z);
//             println!("  Angular Vel:   [{:.4}, {:.4}, {:.4}] rad/s",
//                 spatial.angular_velocity.x, spatial.angular_velocity.y, spatial.angular_velocity.z);

//             println!("\nAerodynamic Data:");
//             println!("----------------");
//             println!("  Alpha:         {:.2}° (angle of attack)", air_data.alpha.to_degrees());
//             println!("  Beta:          {:.2}° (sideslip angle)", air_data.beta.to_degrees());
//             println!("  True Airspeed: {:.2} m/s", air_data.true_airspeed);
//             println!("  Dynamic Press: {:.2} Pa", air_data.dynamic_pressure);

//             println!("\nAircraft Attitude:");
//             println!("-----------------");
//             println!("  Roll (ϕ):      {:.2}°", roll.to_degrees());
//             println!("  Pitch (θ):     {:.2}°", pitch.to_degrees());
//             println!("  Yaw (ψ):       {:.2}°", yaw.to_degrees());

//             println!("\nTrim Control Settings:");
//             println!("---------------------");
//             println!("  Elevator:      {:.4}", controls.elevator);
//             println!("  Aileron:       {:.4}", controls.aileron);
//             println!("  Rudder:        {:.4}", controls.rudder);
//             println!("  Throttle:      {:.4}", controls.power_lever);

//             println!("\nFlight Condition:");
//             println!("----------------");
//             println!("  Trim Type:     Straight and Level");
//             println!("  Target Speed:  50.0 m/s");
//             println!("  Achieved:      {:.2} m/s", spatial.velocity.norm());
//             println!("  Iterations:    {}", tracker.iterations);
//             println!("  Final Cost:    {:.6}", tracker.last_cost);
//             println!("\n=== END OF TRIM REPORT ===");

//             tracker.trim_complete = true;
//             return;
//         }
//     }
// }
