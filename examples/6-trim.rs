use bevy::prelude::*;
use flyer::{
    components::{
        AircraftAeroCoefficients, AircraftGeometry, AircraftType, DragCoefficients,
        FixedStartConfig, FullAircraftConfig, LiftCoefficients, MassModel, NeedsTrim,
        PitchCoefficients, PowerplantConfig, PropulsionConfig, SpatialComponent, StartConfig,
        TrimBounds, TrimCondition, TrimRequest, TrimSolverConfig,
    },
    plugins::{
        EnvironmentPlugin, FullAircraftPlugin, PhysicsPlugin, StartupSequencePlugin,
        TransformationPlugin,
    },
    resources::PhysicsConfig,
    systems::{
        aero_force_system, air_data_system, force_calculator_system, handle_trim_requests,
        physics_integrator_system, trim_aircraft_system,
    },
};
use nalgebra::{Matrix3, Vector3};

#[derive(Resource)]
struct TrimConvergenceTracker {
    last_cost: f64,
    stall_counter: u32,
    iterations: u32,
}

impl Default for TrimConvergenceTracker {
    fn default() -> Self {
        Self {
            last_cost: f64::INFINITY,
            stall_counter: 0,
            iterations: 0,
        }
    }
}

fn main() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default())
        .init_asset::<Image>()
        .init_resource::<Assets<TextureAtlasLayout>>();
    app.insert_resource(Time::<Fixed>::from_hz(1e6)); // 1MHz update rate

    app.add_event::<TrimRequest>();

    // Add required plugins
    app.add_plugins((
        StartupSequencePlugin,
        PhysicsPlugin::with_config(PhysicsConfig::default()),
        TransformationPlugin::default(),
        EnvironmentPlugin::new(),
    ));

    app.insert_resource(TrimConvergenceTracker::default());

    let bounds = TrimBounds {
        elevator_range: (-0.5, 0.5),
        aileron_range: (-0.3, 0.3),
        rudder_range: (-0.3, 0.3),
        throttle_range: (0.1, 0.9),
        alpha_range: (-0.2, 0.2),
        beta_range: (-0.1, 0.1),
        phi_range: (-0.2, 0.2),
        theta_range: (-0.2, 0.2),
    };

    // Configure trim solver
    app.insert_resource(TrimSolverConfig {
        max_iterations: 10000,
        cost_tolerance: 1e-2,
        use_gradient_refinement: true,
        bounds,
    });

    // Create TwinOtter aircraft config
    let aircraft_config = {
        let mut config = FullAircraftConfig::default();

        // Set fixed initial conditions
        config.start_config = StartConfig::Fixed(FixedStartConfig {
            position: Vector3::new(0.0, 0.0, -1000.0), // 1000m altitude
            speed: 100.0,                              // Initial speed 100 m/s
            heading: 0.0,                              // Flying north
        });

        config
    };

    // let aircraft_config = {
    //     // Create a simple test aircraft with reasonable parameters
    //     let mass = 1000.0; // 1000 kg
    //     let wing_area = 16.0; // 16 m^2
    //     let wing_span = 10.0; // 10 m
    //     let mac = 1.6; // 1.6 m mean aerodynamic chord

    //     FullAircraftConfig {
    //         name: "test_aircraft".to_string(),
    //         ac_type: AircraftType::Custom("TestAircraft".to_string()),
    //         mass: MassModel {
    //             mass,
    //             inertia: Matrix3::from_diagonal(&Vector3::new(1000.0, 2000.0, 1500.0)),
    //             inertia_inv: Matrix3::from_diagonal(&Vector3::new(
    //                 1.0 / 1000.0,
    //                 1.0 / 2000.0,
    //                 1.0 / 1500.0,
    //             )),
    //         },
    //         geometry: AircraftGeometry {
    //             wing_area,
    //             wing_span,
    //             mac,
    //         },
    //         aero_coef: AircraftAeroCoefficients {
    //             // Simple but physically plausible coefficients
    //             lift: LiftCoefficients {
    //                 c_l_0: 0.2,
    //                 c_l_alpha: 5.0,
    //                 ..Default::default()
    //             },
    //             drag: DragCoefficients {
    //                 c_d_0: 0.02,
    //                 c_d_alpha2: 0.1,
    //                 ..Default::default()
    //             },
    //             pitch: PitchCoefficients {
    //                 c_m_0: 0.0,
    //                 c_m_alpha: -1.0,
    //                 c_m_q: -10.0,
    //                 c_m_deltae: -1.0,
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         },
    //         propulsion: PropulsionConfig {
    //             engines: vec![PowerplantConfig {
    //                 name: "engine1".to_string(),
    //                 max_thrust: 5000.0,
    //                 min_thrust: 0.0,
    //                 position: Vector3::new(0.0, 0.0, 0.0),
    //                 orientation: Vector3::new(1.0, 0.0, 0.0),
    //                 tsfc: 0.0001,
    //                 spool_up_time: 1.0,
    //                 spool_down_time: 1.0,
    //             }],
    //         },
    //         start_config: StartConfig::Fixed(FixedStartConfig {
    //             position: Vector3::new(0.0, 0.0, -1000.0), // 1000m altitude
    //             speed: 100.0,                              // Initial speed 100 m/s
    //             heading: 0.0,                              // Flying north
    //         }),
    //         task_config: Default::default(),
    //     }
    // };

    app.add_plugins(FullAircraftPlugin::new_single(aircraft_config));

    // Add physics and trim systems
    app.add_systems(
        FixedUpdate,
        (
            air_data_system,
            aero_force_system,
            force_calculator_system,
            physics_integrator_system,
            request_trim,
            handle_trim_requests,
            trim_aircraft_system,
            monitor_trim_convergence,
        )
            .chain(),
    );

    app.run();
}

// System to set up initial aircraft state and trim request
fn request_trim(
    mut trim_requests: EventWriter<TrimRequest>,
    query: Query<(Entity, &SpatialComponent), Added<SpatialComponent>>,
) {
    for (entity, spatial) in query.iter() {
        println!(
            "Requesting trim at altitude: {} meters",
            -spatial.position.z
        );

        trim_requests.send(TrimRequest {
            entity,
            condition: TrimCondition::StraightAndLevel { airspeed: 100.0 },
        });
    }
}

// System to log aircraft state
fn monitor_trim_convergence(
    query: Query<(&SpatialComponent, Option<&NeedsTrim>)>,
    mut tracker: ResMut<TrimConvergenceTracker>,
    time: Res<Time>,
) {
    for (spatial, needs_trim) in query.iter() {
        if let Some(needs_trim) = needs_trim {
            if let Some(ref solver) = needs_trim.solver {
                let current_cost = solver.best_cost;
                tracker.iterations += 1;

                println!(
                    "Iteration {}: Cost = {:.6}, Velocity = {:?}, Alt = {:.1}m",
                    tracker.iterations, current_cost, spatial.velocity, -spatial.position.z
                );

                // Check for non-finite cost
                if !current_cost.is_finite() {
                    println!(
                        "Warning: Cost became non-finite at iteration {}",
                        tracker.iterations
                    );
                    return;
                }

                // Check for stall in convergence
                if (tracker.last_cost - current_cost).abs() < 1e-6 {
                    tracker.stall_counter += 1;
                    if tracker.stall_counter > 5 {
                        println!("Optimization stalled - not making progress");
                        println!("Final state: {:?}", solver.current_state);
                        println!(
                            "Final residuals: {:?}",
                            solver.optimizer.calculate_residuals(&solver.current_state)
                        );
                        return;
                    }
                } else {
                    tracker.stall_counter = 0;
                }

                // Check for convergence
                if current_cost < 1e-2 {
                    println!("Successfully converged at iteration {}", tracker.iterations);
                    println!("Final state: {:?}", solver.current_state);
                    println!(
                        "Time: {:.1}s, Speed: {:.1} m/s, Alt: {:.1}m",
                        time.elapsed_secs(),
                        spatial.velocity.norm(),
                        -spatial.position.z
                    );
                    return;
                }

                tracker.last_cost = current_cost;
            }
        } else if tracker.iterations > 0 {
            // Only print completion once
            // NeedsTrim component removed - convergence achieved
            let (roll, pitch, _) = spatial.attitude.euler_angles();
            println!("\nTrim complete!");
            println!(
                "Final state: Speed = {:.1} m/s, Alt = {:.1}m, Roll = {:.1}°, Pitch = {:.1}°",
                spatial.velocity.norm(),
                -spatial.position.z,
                roll.to_degrees(),
                pitch.to_degrees()
            );
            tracker.iterations = 0; // Reset for next trim operation
            return;
        }
    }
}
