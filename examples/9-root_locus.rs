fn main() {}

// use bevy::prelude::*;
// use flyer::{
//     components::{
//         AircraftControlSurfaces, Force, ForceCategory, FullAircraftConfig, LongitudinalTrimState,
//         PhysicsComponent, PropulsionState, ReferenceFrame, SpatialComponent, TrimState,
//     },
//     plugins::{EnvironmentPlugin, PhysicsPlugin},
//     resources::PhysicsConfig,
//     systems::VirtualPhysics,
// };
// use nalgebra::{Complex, DMatrix, DVector, Matrix4, UnitQuaternion, Vector3, Vector4};
// use std::fs::File;
// use std::io::Write;

// fn main() {
//     println!("Performing root locus analysis of aircraft dynamics");

//     let mut app = App::new();
//     app.add_plugins(MinimalPlugins)
//         .add_plugins(PhysicsPlugin::with_config(PhysicsConfig::default()))
//         .add_plugins(EnvironmentPlugin::new());

//     // Create virtual physics engine
//     let mut virtual_physics = VirtualPhysics::new(&PhysicsConfig::default());

//     // 1. Use a known good trim solution (from 6g-trim.rs)
//     println!("Setting up aircraft at trim condition...");
//     let trim_state = TrimState {
//         longitudinal: LongitudinalTrimState {
//             alpha: -0.08,      // From 6g-trim.rs
//             theta: -0.08,
//             elevator: -0.05,
//             power_lever: 0.75,
//         },
//         ..Default::default()
//     };

//     // 2. Create an aircraft at this trim state
//     let aircraft = setup_aircraft_at_trim(&mut virtual_physics, &trim_state);

//     // 3. Perform the root locus analysis
//     println!("Performing eigenvalue analysis...");

//     // 3a. Analyze longitudinal dynamics
//     println!("\n=== LONGITUDINAL DYNAMICS ANALYSIS ===");
//     let (long_eigenvalues, long_a_matrix) = calculate_longitudinal_eigenvalues(&mut virtual_physics, aircraft);

//     // Output longitudinal results
//     println!("\nLongitudinal State Space A-Matrix:");
//     print_matrix_4x4(&long_a_matrix);

//     println!("\nLongitudinal System Eigenvalues:");
//     print_eigenvalues(&long_eigenvalues);

//     // Save longitudinal results for plotting
//     save_eigenvalues_to_file(&long_eigenvalues, "longitudinal_root_locus.csv");
//     save_matrix_to_file_4x4(&long_a_matrix, "longitudinal_a_matrix.csv");

//     // Generate longitudinal time history for verification
//     generate_time_history(&mut virtual_physics, aircraft, &long_a_matrix, "longitudinal_time_history.csv");

//     // 3b. Analyze lateral-directional dynamics
//     println!("\n=== LATERAL-DIRECTIONAL DYNAMICS ANALYSIS ===");
//     // Create a new physics instance for lateral analysis to avoid state contamination
//     let mut virtual_physics = VirtualPhysics::new(&PhysicsConfig::default());
//     let aircraft = setup_aircraft_at_trim(&mut virtual_physics, &trim_state); // Re-create aircraft

//     let (lat_eigenvalues, lat_a_matrix) = calculate_lateral_eigenvalues(&mut virtual_physics, aircraft);

//     // Output lateral-directional results
//     println!("\nLateral-Directional State Space A-Matrix:");
//     print_matrix_4x4(&lat_a_matrix);

//     println!("\nLateral-Directional System Eigenvalues:");
//     print_eigenvalues(&lat_eigenvalues);

//     // Save lateral-directional results for plotting
//     save_eigenvalues_to_file(&lat_eigenvalues, "lateral_root_locus.csv");
//     save_matrix_to_file_4x4(&lat_a_matrix, "lateral_a_matrix.csv");

//     // Generate lateral-directional time history for verification
//     generate_lateral_time_history(&mut virtual_physics, aircraft, &lat_a_matrix, "lateral_time_history.csv");

//     println!("Root locus analysis complete!");
// }

// fn setup_aircraft_at_trim(virtual_physics: &mut VirtualPhysics, trim_state: &TrimState) -> Entity {
//     let airspeed = 100.0;  // Use a standard airspeed

//     // Set velocity and attitude from trim state
//     let alpha = trim_state.longitudinal.alpha;
//     let theta = trim_state.longitudinal.theta;

//     let velocity = Vector3::new(
//         airspeed * alpha.cos(),
//         0.0,
//         -airspeed * alpha.sin(),
//     );

//     let attitude = UnitQuaternion::from_euler_angles(0.0, theta, 0.0);

//     // Create the spatial component
//     let spatial = SpatialComponent {
//         position: Vector3::new(0.0, 0.0, -1000.0),
//         velocity,
//         attitude,
//         angular_velocity: Vector3::zeros(),
//     };

//     // Create control surfaces from trim values
//     let controls = AircraftControlSurfaces {
//         elevator: trim_state.longitudinal.elevator,
//         aileron: 0.0,
//         rudder: 0.0,
//         power_lever: trim_state.longitudinal.power_lever,
//     };

//     // Spawn the aircraft
//     let aircraft = virtual_physics.spawn_aircraft(
//         &spatial,
//         &PropulsionState::default(),
//         &FullAircraftConfig::default(),
//     );

//     // Set its control surfaces
//     virtual_physics.set_controls(aircraft, &controls);

//     // Run a few steps to stabilize
//     virtual_physics.run_steps(aircraft, 100);

//     println!("Aircraft at trim:");
//     let (spatial, _controls) = virtual_physics.get_state(aircraft);
//     let (forces, moments) = virtual_physics.calculate_forces(aircraft);
//     println!("  Speed: {:.1} m/s", spatial.velocity.norm());
//     println!("  Alpha: {:.2}°", alpha.to_degrees());
//     println!("  Theta: {:.2}°", theta.to_degrees());
//     println!("  Forces: [{:.2}, {:.2}, {:.2}]", forces.x, forces.y, forces.z);
//     println!("  Moments: [{:.2}, {:.2}, {:.2}]", moments.x, moments.y, moments.z);

//     // Return the entity
//     aircraft
// }

// // Define state variables that can be perturbed during linearization
// fn calculate_longitudinal_eigenvalues(virtual_physics: &mut VirtualPhysics, aircraft: Entity) -> (Vec<Complex<f64>>, Matrix4<f64>) {
//     // Create force store to manually preserve forces between steps
//     let mut force_store = (Vec::<Force>::new(), Vec::<flyer::components::Moment>::new());

//     // Run initial steps to stabilize
//     for _ in 0..100 {
//         virtual_physics.run_steps(aircraft, 1);
//     }

//     // Get the stabilized state
//     let (initial_state, _) = virtual_physics.get_state(aircraft);
//     println!("Stabilized state before longitudinal analysis:");
//     println!("  Velocity: [{:.2}, {:.2}, {:.2}]",
//              initial_state.velocity.x,
//              initial_state.velocity.y,
//              initial_state.velocity.z);

//     // Save initial forces from physics component
//     if let Some(physics) = virtual_physics.world.get::<PhysicsComponent>(aircraft) {
//         force_store = (physics.forces.clone(), physics.moments.clone());
//     }

//     // Apply a small perturbation to excite longitudinal modes
//     // Here we apply a small speed change and slight pitch attitude change
//     let mut perturbed_state = initial_state.clone();
//     perturbed_state.velocity.x += 1.0;  // Small speed change
//     virtual_physics.set_state(aircraft, &perturbed_state.velocity, &perturbed_state.attitude);

//     // Now collect state data for eigenvalue analysis
//     println!("Collecting state history for longitudinal system identification...");
//     let num_samples = 300;
//     let mut states = Vec::with_capacity(num_samples);
//     let mut forces = Vec::with_capacity(num_samples);

//     for _ in 0..num_samples {
//         // Get the current state first
//         let (state, _) = virtual_physics.get_state(aircraft);

//         // Prepare the forces we want to add - these enhance coupling for clearer mode identification
//         let vel_pitch_coupling = Force {
//             vector: Vector3::new(0.0, 0.0, -0.05 * state.velocity.x),
//             point: Some(Vector3::new(1.0, 0.0, 0.0)),  // Apply at nose
//             frame: ReferenceFrame::Body,
//             category: ForceCategory::Custom,
//         };

//         let pitch_vel_coupling = Force {
//             vector: Vector3::new(0.1 * state.angular_velocity.y, 0.0, 0.0),
//             point: None,
//             frame: ReferenceFrame::Body,
//             category: ForceCategory::Custom,
//         };

//         // Before physics step, restore forces from previous step
//         if let Some(mut physics) = virtual_physics.world.get_mut::<PhysicsComponent>(aircraft) {
//             // Add our custom forces
//             physics.add_force(vel_pitch_coupling);
//             physics.add_force(pitch_vel_coupling);

//             // Also restore all previous forces
//             for force in &force_store.0 {
//                 physics.add_force(force.clone());
//             }
//             for moment in &force_store.1 {
//                 physics.add_moment(moment.clone());
//             }
//         }

//         // Run a single step
//         virtual_physics.run_steps(aircraft, 1);

//         // After physics step, store the forces for next iteration
//         if let Some(physics) = virtual_physics.world.get::<PhysicsComponent>(aircraft) {
//             force_store = (physics.forces.clone(), physics.moments.clone());
//         }

//         // Record the state
//         let (state, _) = virtual_physics.get_state(aircraft);
//         let (force, moment) = virtual_physics.calculate_forces(aircraft);

//         states.push(state.clone());
//         forces.push((force, moment));
//     }

//     // Build state transition matrix from sequence of states
//     println!("Building longitudinal state-space model...");
//     let a_matrix = build_longitudinal_state_transition_matrix(&states);

//     // Calculate eigenvalues
//     let complex_eigenvalues = a_matrix.complex_eigenvalues();

//     // Convert to Vec<Complex<f64>>
//     let eigenvalues: Vec<Complex<f64>> = complex_eigenvalues.iter().cloned().collect();

//     (eigenvalues, a_matrix)
// }

// // Calculate lateral-directional eigenvalues
// fn calculate_lateral_eigenvalues(virtual_physics: &mut VirtualPhysics, aircraft: Entity) -> (Vec<Complex<f64>>, Matrix4<f64>) {
//     // Create force store to manually preserve forces between steps
//     let mut force_store = (Vec::<Force>::new(), Vec::<flyer::components::Moment>::new());

//     // Run initial steps to stabilize
//     for _ in 0..100 {
//         virtual_physics.run_steps(aircraft, 1);
//     }

//     // Get the stabilized state
//     let (initial_state, _) = virtual_physics.get_state(aircraft);
//     println!("Stabilized state before lateral-directional analysis:");
//     println!("  Velocity: [{:.2}, {:.2}, {:.2}]",
//              initial_state.velocity.x,
//              initial_state.velocity.y,
//              initial_state.velocity.z);
//     println!("  Angular velocity: [{:.4}, {:.4}, {:.4}]",
//              initial_state.angular_velocity.x,
//              initial_state.angular_velocity.y,
//              initial_state.angular_velocity.z);

//     // Save initial forces from physics component
//     if let Some(physics) = virtual_physics.world.get::<PhysicsComponent>(aircraft) {
//         force_store = (physics.forces.clone(), physics.moments.clone());
//     }

//     // Apply a small perturbation to excite lateral-directional modes
//     // Here we apply a small sideslip and roll rate
//     let mut perturbed_state = initial_state.clone();
//     perturbed_state.velocity.y += 2.0;  // Small sideslip
//     perturbed_state.angular_velocity.x += 0.03;  // Small roll rate
//     virtual_physics.set_state(aircraft, &perturbed_state.velocity, &perturbed_state.attitude);

//     // Now collect state data for eigenvalue analysis
//     println!("Collecting state history for lateral-directional system identification...");
//     let num_samples = 300;
//     let mut states = Vec::with_capacity(num_samples);
//     let mut forces = Vec::with_capacity(num_samples);

//     for _ in 0..num_samples {
//         // Get the current state first
//         let (state, _) = virtual_physics.get_state(aircraft);

//         // Prepare the forces we want to add - these enhance coupling for clearer mode identification
//         let sideslip_yaw_coupling = Force {
//             vector: Vector3::new(0.0, 0.0, 0.05 * state.velocity.y),
//             point: Some(Vector3::new(1.0, 0.0, 0.0)),  // Apply at nose
//             frame: ReferenceFrame::Body,
//             category: ForceCategory::Custom,
//         };

//         let roll_yaw_coupling = flyer::components::Moment {
//             vector: Vector3::new(0.0, 0.0, 0.1 * state.angular_velocity.x),
//             frame: ReferenceFrame::Body,
//             category: ForceCategory::Custom,
//         };

//         // Before physics step, restore forces from previous step
//         if let Some(mut physics) = virtual_physics.world.get_mut::<PhysicsComponent>(aircraft) {
//             // Add our custom forces
//             physics.add_force(sideslip_yaw_coupling);
//             physics.add_moment(roll_yaw_coupling);

//             // Also restore all previous forces
//             for force in &force_store.0 {
//                 physics.add_force(force.clone());
//             }
//             for moment in &force_store.1 {
//                 physics.add_moment(moment.clone());
//             }
//         }

//         // Run a single step
//         virtual_physics.run_steps(aircraft, 1);

//         // After physics step, store the forces for next iteration
//         if let Some(physics) = virtual_physics.world.get::<PhysicsComponent>(aircraft) {
//             force_store = (physics.forces.clone(), physics.moments.clone());
//         }

//         // Record the state
//         let (state, _) = virtual_physics.get_state(aircraft);
//         let (force, moment) = virtual_physics.calculate_forces(aircraft);

//         states.push(state.clone());
//         forces.push((force, moment));
//     }

//     // Build state transition matrix from sequence of states
//     println!("Building lateral-directional state-space model...");
//     let a_matrix = build_lateral_state_transition_matrix(&states);

//     // Calculate eigenvalues
//     let complex_eigenvalues = a_matrix.complex_eigenvalues();

//     // Convert to Vec<Complex<f64>>
//     let eigenvalues: Vec<Complex<f64>> = complex_eigenvalues.iter().cloned().collect();

//     (eigenvalues, a_matrix)
// }

// // Build the longitudinal state transition matrix (A-matrix)
// fn build_longitudinal_state_transition_matrix(states: &[SpatialComponent]) -> Matrix4<f64> {
//     // Extract key longitudinal state variables (u, w, q, theta)
//     let u_velocity: Vec<f64> = states.iter().map(|s| s.velocity.x).collect();
//     let w_velocity: Vec<f64> = states.iter().map(|s| s.velocity.z).collect();
//     let q_pitch_rate: Vec<f64> = states.iter().map(|s| s.angular_velocity.y).collect();
//     let theta: Vec<f64> = states.iter().map(|s| s.attitude.euler_angles().1).collect(); // pitch

//     // Calculate derivatives using finite differences
//     let dt = 0.01; // Simulation timestep
//     let mut u_dot = Vec::with_capacity(u_velocity.len() - 1);
//     let mut w_dot = Vec::with_capacity(w_velocity.len() - 1);
//     let mut q_dot = Vec::with_capacity(q_pitch_rate.len() - 1);
//     let mut theta_dot = Vec::with_capacity(theta.len() - 1);

//     for i in 0..u_velocity.len() - 1 {
//         u_dot.push((u_velocity[i+1] - u_velocity[i]) / dt);
//         w_dot.push((w_velocity[i+1] - w_velocity[i]) / dt);
//         q_dot.push((q_pitch_rate[i+1] - q_pitch_rate[i]) / dt);
//         theta_dot.push(q_pitch_rate[i]); // Theta_dot is actually just q
//     }

//     // Remove mean values for better identification
//     let u_mean: f64 = u_velocity.iter().sum::<f64>() / u_velocity.len() as f64;
//     let w_mean: f64 = w_velocity.iter().sum::<f64>() / w_velocity.len() as f64;
//     let q_mean: f64 = q_pitch_rate.iter().sum::<f64>() / q_pitch_rate.len() as f64;
//     let theta_mean: f64 = theta.iter().sum::<f64>() / theta.len() as f64;

//     let u_centered: Vec<f64> = u_velocity.iter().take(u_dot.len())
//         .map(|&v| v - u_mean).collect();
//     let w_centered: Vec<f64> = w_velocity.iter().take(w_dot.len())
//         .map(|&v| v - w_mean).collect();
//     let q_centered: Vec<f64> = q_pitch_rate.iter().take(q_dot.len())
//         .map(|&v| v - q_mean).collect();
//     let theta_centered: Vec<f64> = theta.iter().take(theta_dot.len())
//         .map(|&v| v - theta_mean).collect();

//     // Perform least squares for each row of A
//     let a1_row = solve_least_squares(&u_dot, &[&u_centered, &w_centered, &q_centered, &theta_centered]);
//     let a2_row = solve_least_squares(&w_dot, &[&u_centered, &w_centered, &q_centered, &theta_centered]);
//     let a3_row = solve_least_squares(&q_dot, &[&u_centered, &w_centered, &q_centered, &theta_centered]);

//     // For theta_dot, we know it's exactly equal to q (from kinematics)
//     let a4_row = vec![0.0, 0.0, 1.0, 0.0];

//     // Assemble the A matrix
//     let mut a_matrix = Matrix4::zeros();
//     for i in 0..4 {
//         a_matrix[(0, i)] = a1_row[i];
//         a_matrix[(1, i)] = a2_row[i];
//         a_matrix[(2, i)] = a3_row[i];
//         a_matrix[(3, i)] = a4_row[i];
//     }

//     a_matrix
// }

// // Build the lateral-directional state transition matrix (A-matrix)
// fn build_lateral_state_transition_matrix(states: &[SpatialComponent]) -> Matrix4<f64> {
//     // Extract key lateral-directional state variables (v, p, r, phi)
//     let v_velocity: Vec<f64> = states.iter().map(|s| s.velocity.y).collect(); // lateral velocity
//     let p_roll_rate: Vec<f64> = states.iter().map(|s| s.angular_velocity.x).collect();
//     let r_yaw_rate: Vec<f64> = states.iter().map(|s| s.angular_velocity.z).collect();
//     let phi: Vec<f64> = states.iter().map(|s| s.attitude.euler_angles().0).collect(); // roll angle

//     // Calculate derivatives using finite differences
//     let dt = 0.01; // Simulation timestep
//     let mut v_dot = Vec::with_capacity(v_velocity.len() - 1);
//     let mut p_dot = Vec::with_capacity(p_roll_rate.len() - 1);
//     let mut r_dot = Vec::with_capacity(r_yaw_rate.len() - 1);
//     let mut phi_dot = Vec::with_capacity(phi.len() - 1);

//     for i in 0..v_velocity.len() - 1 {
//         v_dot.push((v_velocity[i+1] - v_velocity[i]) / dt);
//         p_dot.push((p_roll_rate[i+1] - p_roll_rate[i]) / dt);
//         r_dot.push((r_yaw_rate[i+1] - r_yaw_rate[i]) / dt);
//         phi_dot.push(p_roll_rate[i]); // phi_dot is approximately p
//     }

//     // Remove mean values for better identification
//     let v_mean: f64 = v_velocity.iter().sum::<f64>() / v_velocity.len() as f64;
//     let p_mean: f64 = p_roll_rate.iter().sum::<f64>() / p_roll_rate.len() as f64;
//     let r_mean: f64 = r_yaw_rate.iter().sum::<f64>() / r_yaw_rate.len() as f64;
//     let phi_mean: f64 = phi.iter().sum::<f64>() / phi.len() as f64;

//     let v_centered: Vec<f64> = v_velocity.iter().take(v_dot.len())
//         .map(|&v| v - v_mean).collect();
//     let p_centered: Vec<f64> = p_roll_rate.iter().take(p_dot.len())
//         .map(|&v| v - p_mean).collect();
//     let r_centered: Vec<f64> = r_yaw_rate.iter().take(r_dot.len())
//         .map(|&v| v - r_mean).collect();
//     let phi_centered: Vec<f64> = phi.iter().take(phi_dot.len())
//         .map(|&v| v - phi_mean).collect();

//     // Perform least squares for each row of A
//     let a1_row = solve_least_squares(&v_dot, &[&v_centered, &p_centered, &r_centered, &phi_centered]);
//     let a2_row = solve_least_squares(&p_dot, &[&v_centered, &p_centered, &r_centered, &phi_centered]);
//     let a3_row = solve_least_squares(&r_dot, &[&v_centered, &p_centered, &r_centered, &phi_centered]);

//     // For phi_dot, we know it's approximately equal to p (from kinematics)
//     let a4_row = vec![0.0, 1.0, 0.0, 0.0];

//     // Assemble the A matrix
//     let mut a_matrix = Matrix4::zeros();
//     for i in 0..4 {
//         a_matrix[(0, i)] = a1_row[i];
//         a_matrix[(1, i)] = a2_row[i];
//         a_matrix[(2, i)] = a3_row[i];
//         a_matrix[(3, i)] = a4_row[i];
//     }

//     a_matrix
// }

// // Helper function to solve least squares
// fn solve_least_squares(y: &[f64], x_vectors: &[&[f64]]) -> Vec<f64> {
//     let num_samples = y.len();
//     let num_features = x_vectors.len();

//     // Build the design matrix X
//     let mut x_matrix = Vec::with_capacity(num_samples * num_features);
//     for i in 0..num_samples {
//         for j in 0..num_features {
//             x_matrix.push(x_vectors[j][i]);
//         }
//     }

//     // Convert to nalgebra matrices
//     let x = DMatrix::from_row_slice(num_samples, num_features, &x_matrix);
//     let y_vec = DVector::from_row_slice(y);

//     // Solve (X^T X)^(-1) X^T y
//     let xtx = x.transpose() * &x;
//     let xty = x.transpose() * &y_vec;

//     // Safely handle numerical issues
//     let solution = match xtx.try_inverse() {
//         Some(inv) => {
//             let sol = inv * xty;
//             sol.as_slice().to_vec()
//         },
//         None => {
//             // Fallback to SVD or other robust method
//             println!("Warning: Matrix inversion failed, using fallback method");
//             let svd = x.svd(true, true);
//             let sol = svd.solve(&y_vec, 1e-10).unwrap_or(DVector::zeros(num_features));
//             sol.as_slice().to_vec()
//         }
//     };

//     solution
// }

// fn print_matrix_4x4(matrix: &Matrix4<f64>) {
//     for i in 0..4 {
//         print!("  [");
//         for j in 0..4 {
//             print!("{:8.4}", matrix[(i, j)]);
//             if j < 3 { print!(", "); }
//         }
//         println!("]");
//     }
// }

// fn print_eigenvalues(eigenvalues: &[Complex<f64>]) {
//     println!("------------------");

//     for (i, ev) in eigenvalues.iter().enumerate() {
//         let real = ev.re;
//         let imag = ev.im;

//         // Skip tiny imaginary parts (numerical artifacts)
//         let imag_clean = if imag.abs() < 1e-10 { 0.0 } else { imag };

//         // Calculate frequency and damping ratio
//         let magnitude = (real*real + imag_clean*imag_clean).sqrt();
//         let frequency = if imag_clean != 0.0 { imag_clean.abs() / (2.0 * std::f64::consts::PI) } else { 0.0 };
//         let damping = if magnitude > 0.0 { -real / magnitude } else { 1.0 };

//         println!("Eigenvalue {}: {:.4} + {:.4}i", i+1, real, imag_clean);

//         if imag_clean != 0.0 {
//             println!("  Natural frequency: {:.2} Hz", frequency);
//             println!("  Period: {:.2} seconds", 1.0/frequency);
//             println!("  Damping ratio: {:.3}", damping);
//         } else {
//             println!("  Time constant: {:.2} seconds", if real != 0.0 { -1.0/real } else { f64::INFINITY });
//         }

//         // Identify known modes
//         if frequency > 0.01 && frequency < 0.05 && damping < 0.3 {
//             println!("  Mode type: Phugoid (long-period)");
//         } else if frequency > 0.08 && frequency < 1.0 {
//             if damping < 0.7 {
//                 println!("  Mode type: Short-period pitch oscillation");
//             }
//         } else if frequency > 0.1 && frequency < 0.4 && imag_clean != 0.0 {
//             println!("  Mode type: Dutch roll");
//         } else if real < 0.0 && imag_clean.abs() < 1e-6 {
//             if real > -0.1 {
//                 println!("  Mode type: Spiral");
//             } else if real < -0.5 {
//                 println!("  Mode type: Roll subsidence");
//             }
//         }

//         println!();
//     }
// }

// // Function to save eigenvalues to a CSV file for external plotting
// fn save_eigenvalues_to_file(eigenvalues: &[Complex<f64>], filename: &str) {
//     let mut file = File::create(filename).expect("Failed to create eigenvalues file");

//     // Write CSV header
//     writeln!(file, "real,imaginary,magnitude,frequency_hz,damping").expect("Failed to write header");

//     // Write each eigenvalue with its characteristics
//     for ev in eigenvalues {
//         let real = ev.re;
//         let imag = ev.im;
//         let magnitude = (real*real + imag*imag).sqrt();

//         // Calculate frequency in Hz
//         let frequency = if imag.abs() > 1e-10 {
//             imag.abs() / (2.0 * std::f64::consts::PI)
//         } else {
//             0.0
//         };

//         // Calculate damping ratio
//         let damping = if magnitude > 0.0 { -real / magnitude } else { 1.0 };

//         writeln!(file, "{:.6},{:.6},{:.6},{:.6},{:.6}",
//                  real, imag, magnitude, frequency, damping)
//             .expect("Failed to write eigenvalue data");
//     }

//     println!("Eigenvalues saved to {}", filename);
// }

// // Function to save the A matrix to a CSV file
// fn save_matrix_to_file_4x4(matrix: &Matrix4<f64>, filename: &str) {
//     let mut file = File::create(filename).expect("Failed to create matrix file");

//     // Write matrix in CSV format
//     for i in 0..4 {
//         let mut row = String::new();
//         for j in 0..4 {
//             if j > 0 {
//                 row.push_str(",");
//             }
//             row.push_str(&format!("{:.10}", matrix[(i, j)]));
//         }
//         writeln!(file, "{}", row).expect("Failed to write matrix row");
//     }

//     println!("A-matrix saved to {}", filename);
// }

// // Generate time history for longitudinal model verification
// fn generate_time_history(
//     virtual_physics: &mut VirtualPhysics,
//     aircraft: Entity,
//     a_matrix: &Matrix4<f64>,
//     filename: &str
// ) {
//     println!("Generating longitudinal time history for model verification...");

//     // Get initial state
//     let (initial_spatial, _) = virtual_physics.get_state(aircraft);

//     // Reset to initial state
//     virtual_physics.set_state(
//         aircraft,
//         &initial_spatial.velocity,
//         &initial_spatial.attitude
//     );

//     // Create a small disturbance to excite longitudinal modes
//     let mut perturbed_state = initial_spatial.clone();
//     perturbed_state.velocity.x += 2.0;  // Slightly larger disturbance for better visualization
//     virtual_physics.set_state(aircraft, &perturbed_state.velocity, &perturbed_state.attitude);

//     // Run simulation and record states
//     let num_steps = 500;  // Run for 5 seconds (with 0.01s timestep)
//     let mut simulation_states = Vec::with_capacity(num_steps);
//     let mut linearized_states = Vec::with_capacity(num_steps);

//     // Extract initial linearized state
//     let x0 = Vector4::new(
//         perturbed_state.velocity.x - initial_spatial.velocity.x,
//         perturbed_state.velocity.z - initial_spatial.velocity.z,
//         perturbed_state.angular_velocity.y - initial_spatial.angular_velocity.y,
//         perturbed_state.attitude.euler_angles().1 - initial_spatial.attitude.euler_angles().1
//     );

//     // Current linearized state
//     let mut x_linear = x0;

//     // Store initial states
//     simulation_states.push((0.0, perturbed_state));
//     linearized_states.push((0.0, x_linear));

//     // Record states and propagate linear model
//     for step in 1..num_steps {
//         let time = step as f64 * 0.01;  // Assume 0.01s timestep

//         // Run nonlinear simulation
//         virtual_physics.run_steps(aircraft, 1);
//         let (current_spatial, _) = virtual_physics.get_state(aircraft);
//         simulation_states.push((time, current_spatial.clone()));

//         // Propagate linear model using the state transition matrix
//         // x(k+1) = x(k) + A*x(k)*dt  (Forward Euler integration of x_dot = A*x)
//         x_linear = x_linear + a_matrix * x_linear * 0.01;
//         linearized_states.push((time, x_linear));
//     }

//     // Save to file for plotting
//     let mut file = File::create(filename).expect("Failed to create time history file");

//     // Write header
//     writeln!(file, "time,nonlinear_u,nonlinear_w,nonlinear_q,nonlinear_theta,linear_u,linear_w,linear_q,linear_theta").expect("Failed to write header");

//     // Write data for each time step
//     for i in 0..num_steps {
//         let (time, nonlinear_state) = &simulation_states[i];
//         let (_, linear_state) = &linearized_states[i];

//         // Extract nonlinear state variables
//         let nonlinear_u = nonlinear_state.velocity.x;
//         let nonlinear_w = nonlinear_state.velocity.z;
//         let nonlinear_q = nonlinear_state.angular_velocity.y;
//         let nonlinear_theta = nonlinear_state.attitude.euler_angles().1;

//         // Get base values for correct offset
//         let base_u = initial_spatial.velocity.x;
//         let base_w = initial_spatial.velocity.z;
//         let base_q = initial_spatial.angular_velocity.y;
//         let base_theta = initial_spatial.attitude.euler_angles().1;

//         // Extract linear model state (add back the baseline for comparison)
//         let linear_u = linear_state[0] + base_u;
//         let linear_w = linear_state[1] + base_w;
//         let linear_q = linear_state[2] + base_q;
//         let linear_theta = linear_state[3] + base_theta;

//         // Write the row
//         writeln!(
//             file,
//             "{:.4},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
//             time,
//             nonlinear_u, nonlinear_w, nonlinear_q, nonlinear_theta,
//             linear_u, linear_w, linear_q, linear_theta
//         ).expect("Failed to write time history data");
//     }

//     println!("Longitudinal time history saved to {}", filename);
//     println!("The file contains both nonlinear simulation and linearized model responses for comparison.");
// }

// // Generate time history for lateral-directional model verification
// fn generate_lateral_time_history(
//     virtual_physics: &mut VirtualPhysics,
//     aircraft: Entity,
//     a_matrix: &Matrix4<f64>,
//     filename: &str
// ) {
//     println!("Generating lateral-directional time history for model verification...");

//     // Get initial state
//     let (initial_spatial, _) = virtual_physics.get_state(aircraft);

//     // Reset to initial state
//     virtual_physics.set_state(
//         aircraft,
//         &initial_spatial.velocity,
//         &initial_spatial.attitude
//     );

//     // Create a small disturbance to excite lateral-directional modes
//     let mut perturbed_state = initial_spatial.clone();
//     perturbed_state.velocity.y += 2.0;  // Sideslip velocity
//     perturbed_state.angular_velocity.x += 0.05;  // Roll rate
//     virtual_physics.set_state(aircraft, &perturbed_state.velocity, &perturbed_state.attitude);

//     // Run simulation and record states
//     let num_steps = 500;  // Run for 5 seconds (with 0.01s timestep)
//     let mut simulation_states = Vec::with_capacity(num_steps);
//     let mut linearized_states = Vec::with_capacity(num_steps);

//     // Extract initial linearized state (v, p, r, phi)
//     let x0 = Vector4::new(
//         perturbed_state.velocity.y - initial_spatial.velocity.y,
//         perturbed_state.angular_velocity.x - initial_spatial.angular_velocity.x,
//         perturbed_state.angular_velocity.z - initial_spatial.angular_velocity.z,
//         perturbed_state.attitude.euler_angles().0 - initial_spatial.attitude.euler_angles().0
//     );

//     // Current linearized state
//     let mut x_linear = x0;

//     // Store initial states
//     simulation_states.push((0.0, perturbed_state));
//     linearized_states.push((0.0, x_linear));

//     // Record states and propagate linear model
//     for step in 1..num_steps {
//         let time = step as f64 * 0.01;  // Assume 0.01s timestep

//         // Run nonlinear simulation
//         virtual_physics.run_steps(aircraft, 1);
//         let (current_spatial, _) = virtual_physics.get_state(aircraft);
//         simulation_states.push((time, current_spatial.clone()));

//         // Propagate linear model using the state transition matrix
//         // x(k+1) = x(k) + A*x(k)*dt  (Forward Euler integration of x_dot = A*x)
//         x_linear = x_linear + a_matrix * x_linear * 0.01;
//         linearized_states.push((time, x_linear));
//     }

//     // Save to file for plotting
//     let mut file = File::create(filename).expect("Failed to create time history file");

//     // Write header
//     writeln!(file, "time,nonlinear_v,nonlinear_p,nonlinear_r,nonlinear_phi,linear_v,linear_p,linear_r,linear_phi").expect("Failed to write header");

//     // Write data for each time step
//     for i in 0..num_steps {
//         let (time, nonlinear_state) = &simulation_states[i];
//         let (_, linear_state) = &linearized_states[i];

//         // Extract nonlinear state variables
//         let nonlinear_v = nonlinear_state.velocity.y;  // Lateral velocity
//         let nonlinear_p = nonlinear_state.angular_velocity.x;  // Roll rate
//         let nonlinear_r = nonlinear_state.angular_velocity.z;  // Yaw rate
//         let nonlinear_phi = nonlinear_state.attitude.euler_angles().0;  // Roll angle

//         // Get base values for correct offset
//         let base_v = initial_spatial.velocity.y;
//         let base_p = initial_spatial.angular_velocity.x;
//         let base_r = initial_spatial.angular_velocity.z;
//         let base_phi = initial_spatial.attitude.euler_angles().0;

//         // Extract linear model state (add back the baseline for comparison)
//         let linear_v = linear_state[0] + base_v;
//         let linear_p = linear_state[1] + base_p;
//         let linear_r = linear_state[2] + base_r;
//         let linear_phi = linear_state[3] + base_phi;

//         // Write the row
//         writeln!(
//             file,
//             "{:.4},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
//             time,
//             nonlinear_v, nonlinear_p, nonlinear_r, nonlinear_phi,
//             linear_v, linear_p, linear_r, linear_phi
//         ).expect("Failed to write time history data");
//     }

//     println!("Lateral-directional time history saved to {}", filename);
//     println!("The file contains both nonlinear simulation and linearized model responses for comparison.");
// }
