use flyer::{
    components::{FullAircraftConfig, AircraftControlSurfaces, AircraftGeometry, AircraftAeroCoefficients, PowerplantConfig, PowerplantState},
    resources::{PhysicsConfig, EnvironmentConfig, AerodynamicsConfig},
    systems::{calculate_aerodynamic_forces_moments, calculate_engine_outputs, AirDataValues},
};
use nalgebra::{Vector3, UnitQuaternion, Matrix3, DMatrix, DVector};
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Duration;

// --- Bevy/Flyer Imports for Trimming ---
use bevy::prelude::*;
use bevy::app::AppExit;
use flyer::{
    components::{
        AirData,
        FixedStartConfig,
        NeedsTrim,
        PhysicsComponent,
        SpatialComponent,
        StartConfig,
        TrimCondition,
        TrimRequest,
        TrimSolverConfig,
    },
    plugins::{
        EnvironmentPlugin,
        FullAircraftPlugin,
        PhysicsPlugin,
        StartupSequencePlugin,
        TransformationPlugin,
    },
    systems::{
        aero_force_system,
        air_data_system,
        force_calculator_system,
        handle_trim_requests,
        physics_integrator_system,
        propulsion_system,
        trim_aircraft_system,
    },
};

// --- Structs (AircraftState, ControlInputs, StateDerivativesOutput) ---
#[derive(Debug, Clone)]
struct AircraftState {
    u: f64,
    v: f64,
    w: f64,
    p: f64,
    q: f64,
    r: f64,
    phi: f64,
    theta: f64,
    psi: f64,
    pos_x: f64,
    pos_y: f64,
    pos_z: f64,
}

#[derive(Debug, Clone)]
struct ControlInputs {
    elevator: f64,
    aileron: f64,
    rudder: f64,
    throttle: f64,
}

#[derive(Debug, Clone)]
struct StateDerivativesOutput {
    u_dot: f64,
    v_dot: f64,
    w_dot: f64,
    p_dot: f64,
    q_dot: f64,
    r_dot: f64,
    phi_dot: f64,
    theta_dot: f64,
    psi_dot: f64,
    pos_x_dot: f64,
    pos_y_dot: f64,
    pos_z_dot: f64,
}

impl StateDerivativesOutput {
    fn to_vector(&self) -> DVector<f64> {
        DVector::from_vec(vec![
            self.u_dot,
            self.v_dot,
            self.w_dot,
            self.p_dot,
            self.q_dot,
            self.r_dot,
            self.phi_dot,
            self.theta_dot,
            self.psi_dot,
            self.pos_x_dot,
            self.pos_y_dot,
            self.pos_z_dot,
        ])
    }
}

// --- Helper Functions (euler_to_quaternion, etc.) ---
fn euler_to_quaternion(roll: f64, pitch: f64, yaw: f64) -> UnitQuaternion<f64> {
    UnitQuaternion::from_euler_angles(roll, pitch, yaw)
}

fn body_to_inertial_velocity(
    u: f64,
    v: f64,
    w: f64,
    attitude: &UnitQuaternion<f64>,
) -> Vector3<f64> {
    attitude * Vector3::new(u, v, w)
}

fn calculate_euler_rates(
    p: f64,
    q: f64,
    r: f64,
    phi: f64,
    theta: f64,
) -> (f64, f64, f64) {
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    let tan_theta = theta.tan();
    let cos_theta = theta.cos();

    if cos_theta.abs() < 1e-10 {
        return (0.0, 0.0, 0.0);
    }
    let sec_theta = 1.0 / cos_theta;

    let phi_dot = p + q * sin_phi * tan_theta + r * cos_phi * tan_theta;
    let theta_dot = q * cos_phi - r * sin_phi;
    let psi_dot = q * sin_phi * sec_theta + r * cos_phi * sec_theta;

    (phi_dot, theta_dot, psi_dot)
}

fn calculate_body_accel(
    accel_inertial: &Vector3<f64>,
    attitude: &UnitQuaternion<f64>,
    u: f64,
    v: f64,
    w: f64,
    p: f64,
    q: f64,
    r: f64,
) -> (f64, f64, f64) {
    let accel_body = attitude.inverse() * accel_inertial;

    let u_dot = accel_body.x + r * v - q * w;
    let v_dot = accel_body.y + p * w - r * u;
    let w_dot = accel_body.z + q * u - p * v;

    (u_dot, v_dot, w_dot)
}

// --- State Derivative Function: f(x, u) ---
fn calculate_state_derivatives(
    state: &AircraftState,
    controls: &ControlInputs,
    ac_config: &FullAircraftConfig,
    phys_config: &PhysicsConfig,
    env_config: &EnvironmentConfig,
    _aero_config: &AerodynamicsConfig,
) -> StateDerivativesOutput {
    let attitude = euler_to_quaternion(state.phi, state.theta, state.psi);
    let vel_inertial = body_to_inertial_velocity(state.u, state.v, state.w, &attitude);

    let spatial = (
        Vector3::new(state.p, state.q, state.r),
        attitude,
        vel_inertial,
    );

    let control_surfaces = AircraftControlSurfaces {
        elevator: -controls.elevator,
        aileron: -controls.aileron,
        rudder: controls.rudder,
        power_lever: controls.throttle,
        ..Default::default()
    };

    let wind_velocity = Vector3::zeros();
    let altitude = -state.pos_z;
    let rho = 1.225;
    let airspeed_vector_inertial = spatial.2 - wind_velocity;
    let airspeed_vector_body = spatial.1.inverse() * airspeed_vector_inertial;
    let true_airspeed = airspeed_vector_inertial.norm();
    let alpha = if airspeed_vector_body.x.abs() > 1e-6 {
        (airspeed_vector_body.z / airspeed_vector_body.x).atan()
    } else {
        0.0
    };
    let beta = if true_airspeed > 1e-6 {
        (airspeed_vector_body.y / true_airspeed).asin()
    } else {
        0.0
    };
    let q_bar = 0.5 * rho * true_airspeed * true_airspeed;

    let air_data_values = AirDataValues {
        density: rho,
        true_airspeed,
        alpha,
        beta,
        dynamic_pressure: q_bar,
        relative_velocity_body: airspeed_vector_body,
    };

    let mut net_force_inertial = Vector3::new(0.0, 0.0, 9.81);
    let mut net_moment_body = Vector3::zeros();

    for engine_config in ac_config.propulsion.engines.iter() {
        let mut temp_state = PowerplantState::default();
        temp_state.power_lever = controls.throttle;
        temp_state.thrust_fraction = controls.throttle;
        temp_state.running = true;

        let engine_outputs = calculate_engine_outputs(
            engine_config,
            &temp_state,
            rho,
            true_airspeed,
        );

        let force_comp = engine_outputs.force_component;

        net_force_inertial += spatial.1 * force_comp.vector;

        net_moment_body += force_comp.point.unwrap_or_default().cross(&force_comp.vector);
    }

    let (aero_forces_body, aero_moments_body) = calculate_aerodynamic_forces_moments(
        &ac_config.geometry,
        &ac_config.aero_coef,
        &air_data_values,
        &spatial.0,
        &control_surfaces,
    );

    net_force_inertial += spatial.1 * aero_forces_body;
    net_moment_body += aero_moments_body;

    let accel_inertial = net_force_inertial / ac_config.mass.mass;
    let omega = spatial.0;
    let gyro_term = omega.cross(&(ac_config.mass.inertia * omega));
    let angular_accel_body = ac_config.mass.inertia.try_inverse().unwrap_or_else(Matrix3::zeros) * (net_moment_body - gyro_term);

    let (phi_dot, theta_dot, psi_dot) = calculate_euler_rates(state.p, state.q, state.r, state.phi, state.theta);
    let pos_dot = spatial.2;
    let (u_dot, v_dot, w_dot) = calculate_body_accel(&accel_inertial, &spatial.1, state.u, state.v, state.w, state.p, state.q, state.r);

    StateDerivativesOutput {
        u_dot,
        v_dot,
        w_dot,
        p_dot: angular_accel_body.x,
        q_dot: angular_accel_body.y,
        r_dot: angular_accel_body.z,
        phi_dot,
        theta_dot,
        psi_dot,
        pos_x_dot: pos_dot.x,
        pos_y_dot: pos_dot.y,
        pos_z_dot: pos_dot.z,
    }
}

// --- Trim Helpers ---
#[derive(Resource, Default)]
struct TrimResult {
    state: Option<SpatialComponent>,
    controls: Option<AircraftControlSurfaces>,
    completed: bool,
}

fn setup_trim_request(
    mut trim_requests: EventWriter<TrimRequest>,
    query: Query<Entity, Added<FullAircraftConfig>>,
) {
    for entity in query.iter() {
        println!("Sending initial trim request for entity {:?}", entity);
        trim_requests.send(TrimRequest {
            entity,
            condition: TrimCondition::StraightAndLevel { airspeed: 60.0 },
        });
    }
}

fn check_trim_completion(
    mut result: ResMut<TrimResult>,
    query: Query<
        (Entity, &SpatialComponent, &AircraftControlSurfaces),
        (With<FullAircraftConfig>, Without<NeedsTrim>),
    >,
    needs_trim_query: Query<(), (With<FullAircraftConfig>, With<NeedsTrim>)>
) {
    if result.completed { return; }

    if !query.is_empty() && needs_trim_query.is_empty() {
        if let Some((entity, spatial, controls)) = query.iter().next() {
            println!("Trim completed for entity {:?}", entity);
            result.state = Some(spatial.clone());
            result.controls = Some(controls.clone());
            result.completed = true;
        }
    } else if !needs_trim_query.is_empty() {
        // Still trimming
    } else if query.is_empty() && needs_trim_query.is_empty() {
        // Potentially state where aircraft exists but trim hasn't started or finished yet
    }
}

// --- Trim Function ---
/// Performs aircraft trimming using a temporary Bevy app instance.
fn find_trim_condition(
    base_ac_config: &FullAircraftConfig,
    initial_speed: f64,
    initial_altitude: f64,
    physics_hz: f64,
    max_duration_secs: f64,
) -> Result<(AircraftState, ControlInputs), String> {
    println!(
        "\n--- Starting Trim Simulation (Target: {:.1} m/s @ {:.1} m Alt) ---",
        initial_speed, initial_altitude
    );

    let mut app = App::new();

    // Minimal Bevy setup
    app.add_plugins(MinimalPlugins);
    // Configure FixedUpdate schedule
    app.insert_resource(Time::<Fixed>::from_hz(physics_hz));


    // --- Flyer Library Setup ---
    app.insert_resource(TrimSolverConfig::default());
    app.add_event::<TrimRequest>(); // Manually add Trim Event

    app.add_plugins((
        EnvironmentPlugin::new(), // Use default environment
        PhysicsPlugin::with_config(PhysicsConfig {
            timestep: 1.0 / physics_hz,
            ..default()
        }),
        StartupSequencePlugin, // Needed for initial setup
        TransformationPlugin::default(), // Needed for spatial transforms
    ));

    // --- Aircraft Configuration ---
    let start_config = StartConfig::Fixed(FixedStartConfig {
        position: Vector3::new(0.0, 0.0, -initial_altitude), // NED convention
        speed: initial_speed,
        heading: 0.0, // Straight flight
    });
    let mut aircraft_plugin_config = base_ac_config.clone();
    aircraft_plugin_config.start_config = start_config;
    // Add the aircraft using the plugin (simpler than manual entity spawning)
    app.add_plugins(FullAircraftPlugin::new_single(aircraft_plugin_config));

    // --- Trim Control & Result Handling ---
    app.init_resource::<TrimResult>(); // Initialize the result storage
    app.add_systems(PostStartup, setup_trim_request); // Trigger trim after aircraft is added

    // Manually add physics and trim systems like in 6-trim.rs
    app.add_systems(
        FixedUpdate,
        (
            // Core physics loop (ensure order is reasonable)
            // Note: PhysicsPlugin *might* already add these, but 6-trim adds them manually too.
            // Redundant addition is usually okay in Bevy.
            air_data_system,
            aero_force_system,
            propulsion_system,
            force_calculator_system,
            physics_integrator_system,
            // Trim system runs after physics integration
            trim_aircraft_system,
            // Check completion *after* trim system might remove NeedsTrim
            check_trim_completion,
        )
            .chain(),
    );
    // Manually add trim event handler
    app.add_systems(Update, handle_trim_requests);


    // --- Run Trim Simulation Loop ---
    let max_steps = (max_duration_secs * physics_hz).ceil() as usize;
    println!("Max trim steps: {}", max_steps);
    let mut step_count = 0;

    // Initial update to apply PostStartup systems etc.
    app.update();

    loop {
        // 1. Advance the fixed timestep clock
        app.world_mut().resource_mut::<Time<Fixed>>().advance_by(Duration::from_secs_f64(1.0 / physics_hz));

        // 2. Run all due schedules (including FixedUpdate if time threshold met, and Update)
        app.update();

        step_count += 1;

        let trim_result = app.world().resource::<TrimResult>();
        if trim_result.completed {
            println!("Trim converged in {} steps.", step_count);
            let spatial = trim_result.state.as_ref().unwrap();
            let controls = trim_result.controls.as_ref().unwrap();

            // --- Convert Bevy components to linearization structs ---
            // Attitude (Quaternion to Euler RPY in radians)
            let (roll, pitch, yaw) = spatial.attitude.euler_angles(); // Returns (roll, pitch, yaw)

            // Velocity (Inertial to Body frame)
            let vel_inertial = spatial.velocity;
            let vel_body = spatial.attitude.inverse() * vel_inertial;

            // Angular Velocity (already in body frame, rad/s)
            let ang_vel_body = spatial.angular_velocity;

            // Position (Inertial, NED)
            // Use a default position as it's not critical for linearization itself
            let pos_x = 0.0;
            let pos_y = 0.0;
            let pos_z = -initial_altitude;

            // Create AircraftState
            let x_trim = AircraftState {
                u: vel_body.x,
                v: vel_body.y,
                w: vel_body.z,
                p: ang_vel_body.x,
                q: ang_vel_body.y,
                r: ang_vel_body.z,
                phi: roll,
                theta: pitch,
                psi: yaw,
                pos_x,
                pos_y,
                pos_z,
            };

            // Create ControlInputs
            // IMPORTANT: calculate_state_derivatives inverts elevator/aileron internally.
            // So, we provide the direct values from the trim solver here.
            let u_trim = ControlInputs {
                elevator: controls.elevator,
                aileron: controls.aileron,
                rudder: controls.rudder,
                throttle: controls.power_lever, // Assuming power_lever is 0-1
            };

            return Ok((x_trim, u_trim));
        }

        if step_count >= max_steps {
            println!("Trim did not converge within {} steps.", max_steps);
            return Err("Trim simulation timed out".to_string());
        }
    }
}

fn main() {
    println!("--- Aircraft Linearization Example ---");

    // --- 1. Load Configuration ---
    let ac_config = FullAircraftConfig::default(); // Use default Twin Otter for now
    let phys_config = PhysicsConfig::default();
    let env_config = EnvironmentConfig::default();
    let aero_config = AerodynamicsConfig { min_airspeed_threshold: 0.1 };

    // --- 2. Find Trim Condition ---
    let initial_speed_ms = 80.0; // Example: Target speed for trim
    let initial_altitude_m = 1000.0; // Example: Target altitude for trim
    let physics_hz = 100.0; // Simulation frequency for trim
    let trim_timeout_secs = 60.0; // Max time for trim to converge

    let (x_trim, u_trim) = match find_trim_condition(
        &ac_config,
        initial_speed_ms,
        initial_altitude_m,
        physics_hz,
        trim_timeout_secs,
    ) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error finding trim condition: {}", e);
            return; // Exit if trim fails
        }
    };

    // Convert trim control angles from radians to degrees for printing
    let elevator_deg = u_trim.elevator.to_degrees();
    let aileron_deg = u_trim.aileron.to_degrees();
    let rudder_deg = u_trim.rudder.to_degrees();

    // Convert trim state angles/rates from radians to degrees for printing
    let phi_deg = x_trim.phi.to_degrees();
    let theta_deg = x_trim.theta.to_degrees();
    let psi_deg = x_trim.psi.to_degrees();
    let p_deg = x_trim.p.to_degrees();
    let q_deg = x_trim.q.to_degrees();
    let r_deg = x_trim.r.to_degrees();

    println!("\n--- Trim Condition Found ---");
    println!(
        "State (x_trim): u={:.2} v={:.2} w={:.2} (m/s) | p={:.2} q={:.2} r={:.2} (deg/s) | phi={:.2} theta={:.2} psi={:.2} (deg) | alt={:.1} m",
        x_trim.u, x_trim.v, x_trim.w, p_deg, q_deg, r_deg, phi_deg, theta_deg, psi_deg, -x_trim.pos_z
    );
    println!(
        "Controls (u_trim): Elevator={:.2} Aileron={:.2} Rudder={:.2} (deg) | Throttle={:.2}",
        elevator_deg, aileron_deg, rudder_deg, u_trim.throttle
    );
    println!("----------------------------\n");

    // --- 3. Finite Difference Setup ---
    let delta = 1e-6;
    let n_states = 12;
    let n_inputs = 4;

    let mut a_matrix = DMatrix::<f64>::zeros(n_states, n_states);
    let mut b_matrix = DMatrix::<f64>::zeros(n_states, n_inputs);

    // Calculate derivatives at the trim point
    let f0 = calculate_state_derivatives(
        &x_trim,
        &u_trim,
        &ac_config,
        &phys_config,
        &env_config,
        &aero_config,
    );
    let f0_vec = f0.to_vector();

     // Check if derivatives are near zero at trim (should be for forces/moments)
    println!("Derivatives at Trim (f0):");
    println!("  Vel Rates (u_dot,v_dot,w_dot): {:.4e}, {:.4e}, {:.4e}", f0.u_dot, f0.v_dot, f0.w_dot);
    println!("  Ang Rates (p_dot,q_dot,r_dot): {:.4e}, {:.4e}, {:.4e}", f0.p_dot, f0.q_dot, f0.r_dot);
    // Kinematic rates won't necessarily be zero
    println!("  Eul Rates (phi_dot,theta_dot,psi_dot): {:.4e}, {:.4e}, {:.4e}", f0.phi_dot, f0.theta_dot, f0.psi_dot);
    println!("  Pos Rates (x_dot,y_dot,z_dot): {:.4e}, {:.4e}, {:.4e}\n", f0.pos_x_dot, f0.pos_y_dot, f0.pos_z_dot);

    // --- 4. Calculate A Matrix (Jacobian wrt State) ---
    println!("Calculating A Matrix...");
    for i in 0..n_states {
        let mut x_perturbed = x_trim.clone();
        match i {
            0 => x_perturbed.u += delta,
            1 => x_perturbed.v += delta,
            2 => x_perturbed.w += delta,
            3 => x_perturbed.p += delta,
            4 => x_perturbed.q += delta,
            5 => x_perturbed.r += delta,
            6 => x_perturbed.phi += delta,
            7 => x_perturbed.theta += delta,
            8 => x_perturbed.psi += delta,
            9 => x_perturbed.pos_x += delta,
            10 => x_perturbed.pos_y += delta,
            11 => x_perturbed.pos_z += delta,
            _ => unreachable!(),
        }

        let fi = calculate_state_derivatives(
            &x_perturbed,
            &u_trim,
            &ac_config,
            &phys_config,
            &env_config,
            &aero_config,
        );
        let fi_vec = fi.to_vector();

        let a_column = (fi_vec - &f0_vec) / delta;
        a_matrix.set_column(i, &a_column);
    }

    // --- 5. Calculate B Matrix (Jacobian wrt Control Inputs) ---
    println!("Calculating B Matrix...");
    for j in 0..n_inputs {
        let mut u_perturbed = u_trim.clone();
        match j {
            0 => u_perturbed.elevator += delta,
            1 => u_perturbed.aileron += delta,
            2 => u_perturbed.rudder += delta,
            3 => u_perturbed.throttle += delta,
            _ => unreachable!(),
        }

        let fj = calculate_state_derivatives(
            &x_trim,
            &u_perturbed,
            &ac_config,
            &phys_config,
            &env_config,
            &aero_config,
        );
        let fj_vec = fj.to_vector();

        let b_column = (fj_vec - &f0_vec) / delta;
        b_matrix.set_column(j, &b_column);
    }

    println!("");
    println!("--- Linearized System Matrices ---");
    println!("A Matrix (dx/dt = Ax + Bu):\n\n{}\n", a_matrix);
    println!("B Matrix (dx/dt = Ax + Bu):\n\n{}\n", b_matrix);

    // --- 6. Save Matrices to CSV File ---
    if let Err(e) = save_matrices_to_csv(&a_matrix, &b_matrix) {
        eprintln!("Error saving matrices to CSV: {}", e);
    }
}

// --- Helper Functions (write_matrix_csv, save_matrices_to_csv) ---
fn write_matrix_csv<W: Write, R: nalgebra::Dim, C: nalgebra::Dim, S: nalgebra::Storage<f64, R, C>>(
    writer: &mut BufWriter<W>,
    matrix: &nalgebra::Matrix<f64, R, C, S>,
    title: &str,
    row_labels: &[&str],
    col_labels: &[&str],
) -> std::io::Result<()> {
    writeln!(writer, "{}", title)?;
    write!(writer, "State")?;
    for label in col_labels {
        write!(writer, ",{}", label)?;
    }
    writeln!(writer)?;

    for i in 0..matrix.nrows() {
        write!(writer, "{}", row_labels[i])?;
        for j in 0..matrix.ncols() {
            write!(writer, ",{:.8e}", matrix[(i, j)])?;
        }
        writeln!(writer)?;
    }
    writeln!(writer)?;
    Ok(())
}

fn save_matrices_to_csv(
    a_matrix: &DMatrix<f64>,
    b_matrix: &DMatrix<f64>,
) -> std::io::Result<()> {
    let state_labels = [
        "u (m/s)", "v (m/s)", "w (m/s)",
        "p (rad/s)", "q (rad/s)", "r (rad/s)",
        "phi (rad)", "theta (rad)", "psi (rad)",
        "pos_x (m)", "pos_y (m)", "pos_z (m)",
    ];
    let input_labels = ["elevator", "aileron", "rudder", "throttle"];

    let file = File::create("linearized_matrices.csv")?;
    let mut writer = BufWriter::new(file);

    write_matrix_csv(
        &mut writer,
        a_matrix,
        "A Matrix (dx/dt = Ax + Bu)",
        &state_labels,
        &state_labels,
    )?;

    write_matrix_csv(
        &mut writer,
        b_matrix,
        "B Matrix (dx/dt = Ax + Bu)",
        &state_labels,
        &input_labels,
    )?;

    writer.flush()?;
    println!("Matrices saved to linearized_matrices.csv");
    Ok(())
}
