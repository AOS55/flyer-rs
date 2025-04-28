use bevy::app::AppExit;
use bevy::prelude::*; // Keep this import

// --- Flyer Imports ---
// Core Components needed generally
use flyer::components::{
    // AirData, // Removed unused import
    AircraftControlSurfaces,
    FixedStartConfig,
    // PropulsionState, // Removed unused import (used internally in calculations)
    Force,
    ForceCategory,
    FullAircraftConfig,
    Moment,
    NeedsTrim,
    ReferenceFrame, // Needed for calculate_net_forces_moments
    // PhysicsComponent, // Removed unused import
    SpatialComponent,
    StartConfig, // TrimCondition, // Removed unused import
    TrimRequest,
    TrimSolverConfig,
};
// Components/Structs specifically for calculation functions
// Removed unused: AircraftGeometry, AircraftAeroCoefficients, PowerplantConfig
use flyer::components::PowerplantState; // Still needed for temp state construction

// Plugins
use flyer::plugins::{
    EnvironmentPlugin, FullAircraftPlugin, PhysicsPlugin, StartupSequencePlugin,
    TransformationPlugin,
};
// Resources
use flyer::resources::{AerodynamicsConfig, EnvironmentConfig, EnvironmentModel, PhysicsConfig};
// Systems - Crucially, the *pure calculation functions*
use flyer::systems::{
    // Physics loop systems (still run by Bevy)
    aero_force_system,
    air_data_system,
    calculate_aerodynamic_forces_moments,
    // *** Pure Calculation Functions to Reuse ***
    calculate_air_data,
    calculate_engine_outputs,
    calculate_net_forces_moments,
    force_calculator_system,
    // Trim systems (still run by Bevy)
    handle_trim_requests,
    physics_integrator_system,
    propulsion_system,
    trim_aircraft_system,
    AirDataValues,
    EngineOutputs,
};

// --- Nalgebra/Std Imports ---
// Removed unused: Matrix3
use nalgebra::{DMatrix, DVector, UnitQuaternion, Vector3};
// Removed unused: File, BufWriter, Write, Path
use std::fs::File;
use std::io::{BufWriter, Write}; // Keep BufWriter, Write // Keep File

// --- Structs for Linearization (State, Controls, Derivatives) - Definitions needed ---
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
    // *** Restore implementation ***
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

// --- Helper Functions for Linearization - Restore implementations ---
fn euler_to_quaternion(roll: f64, pitch: f64, yaw: f64) -> UnitQuaternion<f64> {
    UnitQuaternion::from_euler_angles(roll, pitch, yaw)
}

fn calculate_euler_rates(p: f64, q: f64, r: f64, phi: f64, theta: f64) -> (f64, f64, f64) {
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    let tan_theta = theta.tan();
    let cos_theta = theta.cos();

    if cos_theta.abs() < 1e-10 {
        warn!("Near singularity in Euler rate calculation (theta close to +/- 90 deg)");
        return (p, 0.0, 0.0);
    }
    let sec_theta = 1.0 / cos_theta;

    let phi_dot = p + q * sin_phi * tan_theta + r * cos_phi * tan_theta;
    let theta_dot = q * cos_phi - r * sin_phi;
    let psi_dot = q * sin_phi * sec_theta + r * cos_phi * sec_theta;

    (phi_dot, theta_dot, psi_dot)
}

// --- NEW State Derivative Function using Flyer's Pure Functions ---
fn calculate_flyer_state_derivatives(
    state: &AircraftState,
    controls: &ControlInputs,
    ac_config: &FullAircraftConfig,
    phys_config: &PhysicsConfig,
    env_model: &EnvironmentModel,
) -> StateDerivativesOutput {
    // 1. Reconstruct necessary Bevy-like structs from linearization state/controls
    let attitude = euler_to_quaternion(state.phi, state.theta, state.psi);
    let vel_body = Vector3::new(state.u, state.v, state.w);
    let vel_inertial = attitude * vel_body;
    let ang_vel_body = Vector3::new(state.p, state.q, state.r);
    let position_inertial = Vector3::new(state.pos_x, state.pos_y, state.pos_z);

    let current_spatial = SpatialComponent {
        position: position_inertial,
        velocity: vel_inertial,
        attitude,
        angular_velocity: ang_vel_body,
        // Removed non-existent fields: acceleration, angular_acceleration
    };

    let current_controls = AircraftControlSurfaces {
        elevator: controls.elevator,
        aileron: controls.aileron,
        rudder: controls.rudder,
        power_lever: controls.throttle,
        // Removed non-existent fields: flaps, slats, speed_brake
    };

    // 2. Call Flyer's Pure Calculation Functions

    // 2a. Air Data
    let altitude = -state.pos_z;
    let density = env_model.get_density_at_altitude(altitude);
    let wind_inertial = Vector3::zeros();
    let air_data_values: AirDataValues = calculate_air_data(
        &current_spatial.velocity,
        &current_spatial.attitude,
        &wind_inertial,
        density,
        0.1, // Use a default threshold or get from AerodynamicsConfig if needed/available
    );

    // 2b. Aerodynamics
    let (aero_f_body, aero_m_body) = calculate_aerodynamic_forces_moments(
        &ac_config.geometry,
        &ac_config.aero_coef,
        &air_data_values,
        &current_spatial.angular_velocity,
        &current_controls,
    );

    let mut external_forces: Vec<Force> = Vec::new();
    let mut external_moments: Vec<Moment> = Vec::new();

    external_forces.push(Force {
        vector: aero_f_body,
        category: ForceCategory::Aerodynamic,
        frame: ReferenceFrame::Body,
        point: None,
    });
    external_moments.push(Moment {
        vector: aero_m_body,
        category: ForceCategory::Aerodynamic,
        frame: ReferenceFrame::Body,
    });

    // 2c. Propulsion
    for engine_config in ac_config.propulsion.engines.iter() {
        let temp_engine_state = PowerplantState {
            power_lever: controls.throttle,
            thrust_fraction: controls.throttle,
            running: controls.throttle > 0.01,
            fuel_flow: 0.0,
            // Removed non-existent field: rpm
        };

        let engine_outputs: EngineOutputs = calculate_engine_outputs(
            engine_config,
            &temp_engine_state,
            air_data_values.density,
            air_data_values.true_airspeed,
        );

        if engine_outputs.force_component.vector.norm_squared() > 1e-9 {
            external_forces.push(engine_outputs.force_component);
        }
        // Removed logic for non-existent engine_outputs.moment_component
    }

    // 2d. Net Forces/Moments (Includes Gravity)
    let (net_force_inertial, net_moment_body, _grav_force_body) = calculate_net_forces_moments(
        &external_forces,
        &external_moments,
        &current_spatial.attitude,
        ac_config.mass.mass,
        &phys_config.gravity,
    );

    // 3. Calculate State Derivatives from Forces/Moments

    // 3a. Linear Acceleration -> u_dot, v_dot, w_dot
    let accel_inertial = net_force_inertial / ac_config.mass.mass;
    let accel_body = current_spatial.attitude.inverse() * accel_inertial;
    let lin_accel_body_frame = accel_body - current_spatial.angular_velocity.cross(&vel_body);

    // 3b. Angular Acceleration -> p_dot, q_dot, r_dot
    let inertia = ac_config.mass.inertia;
    let inertia_inv = ac_config.mass.inertia_inv;
    let omega_body = current_spatial.angular_velocity;
    let gyro_term = omega_body.cross(&(inertia * omega_body));
    let angular_accel_body = inertia_inv * (net_moment_body - gyro_term);

    // 3c. Kinematic Rates (Euler Angles & Position) -> phi_dot, theta_dot, psi_dot, pos_dots
    let (phi_dot, theta_dot, psi_dot) =
        calculate_euler_rates(state.p, state.q, state.r, state.phi, state.theta);
    let pos_dot_inertial = current_spatial.velocity;

    // 4. Assemble Output Struct
    StateDerivativesOutput {
        u_dot: lin_accel_body_frame.x,
        v_dot: lin_accel_body_frame.y,
        w_dot: lin_accel_body_frame.z,
        p_dot: angular_accel_body.x,
        q_dot: angular_accel_body.y,
        r_dot: angular_accel_body.z,
        phi_dot,
        theta_dot,
        psi_dot,
        pos_x_dot: pos_dot_inertial.x,
        pos_y_dot: pos_dot_inertial.y,
        pos_z_dot: pos_dot_inertial.z,
    }
}

// --- Bevy Resources and Systems ---

#[derive(Component)]
struct TrackedAircraft;

#[derive(Resource, Default)]
struct LinearizationResult {
    a_matrix: Option<DMatrix<f64>>,
    b_matrix: Option<DMatrix<f64>>,
    x_trim: Option<AircraftState>,
    u_trim: Option<ControlInputs>,
    completed: bool,
}

fn main() {
    let mut app = App::new();
    let physics_hz = 100.0;

    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::log::LogPlugin::default());

    app.insert_resource(Time::<Fixed>::from_hz(physics_hz));

    // --- Flyer Library Setup ---
    app.insert_resource(TrimSolverConfig::default());
    app.insert_resource(EnvironmentConfig::default());
    // *** Correct AerodynamicsConfig initialization ***
    app.insert_resource(AerodynamicsConfig {
        min_airspeed_threshold: 0.1,
    });
    app.add_plugins(EnvironmentPlugin::new());
    app.add_event::<TrimRequest>();
    let phys_config = PhysicsConfig {
        timestep: 1.0 / physics_hz,
        ..default()
    };
    app.insert_resource(phys_config.clone());
    app.add_plugins((
        PhysicsPlugin::with_config(phys_config),
        StartupSequencePlugin,
        TransformationPlugin::default(),
    ));

    // --- Aircraft Configuration ---
    let aircraft_config_data = FullAircraftConfig::f16c();
    let initial_speed = 150.0;
    let initial_altitude_m = 500.0;
    // *** Correct FixedStartConfig initialization ***
    let start_config = StartConfig::Fixed(FixedStartConfig {
        position: Vector3::new(0.0, 0.0, -initial_altitude_m), // NED convention
        speed: initial_speed,
        heading: 0.0, // Straight flight
    });
    let mut aircraft_plugin_config = aircraft_config_data.clone();
    aircraft_plugin_config.start_config = start_config;
    app.add_plugins(FullAircraftPlugin::new_single(aircraft_plugin_config));

    app.init_resource::<LinearizationResult>();

    // --- System Scheduling ---
    app.add_systems(
        FixedUpdate,
        (
            air_data_system,
            aero_force_system,
            propulsion_system,
            force_calculator_system,
            physics_integrator_system,
            trim_aircraft_system,
            check_trim_completion_and_linearize.after(trim_aircraft_system),
        )
            .chain(),
    );
    app.add_systems(Update, handle_trim_requests);
    app.add_systems(PostStartup, setup_tracking_and_trigger_trim);

    println!("Starting Trim and Linearization (using flyer's pure functions)...");
    app.run();
}

// System to add marker and send the initial trim request
fn setup_tracking_and_trigger_trim(
    mut commands: Commands,
    mut trim_requests: EventWriter<TrimRequest>,
    query: Query<(Entity, &FullAircraftConfig), Added<FullAircraftConfig>>,
) {
    if let Ok((entity, aircraft_config)) = query.get_single() {
        println!(
            "Found aircraft entity {:?}, adding TrackedAircraft marker.",
            entity
        );
        commands.entity(entity).insert(TrackedAircraft);

        let target_airspeed = match &aircraft_config.start_config {
            StartConfig::Fixed(fsc) => fsc.speed,
            _ => {
                warn!("Aircraft start config is not Fixed type, using default 80.0 m/s for trim request");
                80.0
            }
        };

        println!(
            "Sending TrimRequest for StraightLevel @ {:.1} m/s to entity {:?}",
            target_airspeed, entity
        );
        trim_requests.send(TrimRequest {
            entity,
            // Use the correct condition name from your library if different
            condition: flyer::components::TrimCondition::StraightAndLevel {
                airspeed: target_airspeed,
            },
        });
    } else {
        error!("Could not find unique aircraft entity after startup to send trim request.");
    }
}

// System that runs AFTER trim_aircraft_system
fn check_trim_completion_and_linearize(
    mut linearization_result: ResMut<LinearizationResult>,
    query: Query<
        (
            Entity,
            &SpatialComponent,
            &AircraftControlSurfaces,
            &FullAircraftConfig,
            Option<&NeedsTrim>,
        ),
        With<TrackedAircraft>,
    >,
    phys_config: Res<PhysicsConfig>,
    env_model: Res<EnvironmentModel>,
    mut exit: EventWriter<AppExit>,
    time: Res<Time>,
) {
    if linearization_result.completed {
        return;
    }

    if let Ok((entity, spatial, controls, ac_config, needs_trim_opt)) = query.get_single() {
        let is_trim_complete = match needs_trim_opt {
            Some(needs_trim) => needs_trim.stage == flyer::components::TrimStage::Complete,
            None => true,
        };
        if !is_trim_complete {
            return;
        }

        // --- Trim is Complete - Proceed ---
        // *** Correct typo: elapsed_secs_f64 ***
        println!(
            "\n--- Trim Condition Applied at t={:.2}s for Entity {:?} ---",
            time.elapsed_secs_f64(),
            entity
        );

        let (roll, pitch, yaw) = spatial.attitude.euler_angles();
        let vel_body = spatial.attitude.inverse() * spatial.velocity;
        let ang_vel_body = spatial.angular_velocity;
        let x_trim = AircraftState {
            /* ... initialize fields ... */
            u: vel_body.x,
            v: vel_body.y,
            w: vel_body.z,
            p: ang_vel_body.x,
            q: ang_vel_body.y,
            r: ang_vel_body.z,
            phi: roll,
            theta: pitch,
            psi: yaw,
            pos_x: spatial.position.x,
            pos_y: spatial.position.y,
            pos_z: spatial.position.z,
        };
        let u_trim = ControlInputs {
            /* ... initialize fields ... */
            elevator: controls.elevator,
            aileron: controls.aileron,
            rudder: controls.rudder,
            throttle: controls.power_lever,
        };

        println!(
             "Trim State (x_trim): u={:.3} v={:.3e} w={:.3e} (m/s) | p={:.3e} q={:.3e} r={:.3e} (rad/s) | phi={:.3e} theta={:.3} psi={:.3e} (rad) | pos=({:.1}, {:.1}, {:.1})m",
             x_trim.u, x_trim.v, x_trim.w, x_trim.p, x_trim.q, x_trim.r, x_trim.phi, x_trim.theta, x_trim.psi, x_trim.pos_x, x_trim.pos_y, x_trim.pos_z
         );
        println!(
             "Trim Controls (u_trim): Elevator={:.6e} Aileron={:.6e} Rudder={:.6e} (rad/def?) | Throttle={:.6e}",
             u_trim.elevator, u_trim.aileron, u_trim.rudder, u_trim.throttle
         );
        println!("-----------------------------------------------------\n");

        // --- Finite Difference ---
        let delta = 1e-6;
        let n_states = 12;
        let n_inputs = 4;
        let mut a_matrix = DMatrix::<f64>::zeros(n_states, n_states);
        let mut b_matrix = DMatrix::<f64>::zeros(n_states, n_inputs);

        let f0 = calculate_flyer_state_derivatives(
            &x_trim,
            &u_trim,
            ac_config,
            &phys_config,
            &env_model,
        );
        let f0_vec = f0.to_vector();

        println!("Derivatives at Trim Point (f0 - using flyer functions):");
        println!(
            "  [u_dot, v_dot, w_dot] = [{:.3e}, {:.3e}, {:.3e}]",
            f0.u_dot, f0.v_dot, f0.w_dot
        );
        println!(
            "  [p_dot, q_dot, r_dot] = [{:.3e}, {:.3e}, {:.3e}]",
            f0.p_dot, f0.q_dot, f0.r_dot
        );
        println!(
            "  [phi_dot, theta_dot, psi_dot] = [{:.3e}, {:.3e}, {:.3e}]",
            f0.phi_dot, f0.theta_dot, f0.psi_dot
        );
        println!(
            "  [pos_x_dot, pos_y_dot, pos_z_dot] = [{:.3e}, {:.3e}, {:.3e}]\n",
            f0.pos_x_dot, f0.pos_y_dot, f0.pos_z_dot
        );

        println!("Calculating A Matrix (Jacobian wrt State)...");
        for i in 0..n_states {
            let mut x_perturbed = x_trim.clone();
            // *** Restore perturbation logic ***
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
            let fi = calculate_flyer_state_derivatives(
                &x_perturbed,
                &u_trim,
                ac_config,
                &phys_config,
                &env_model,
            );
            let fi_vec = fi.to_vector();
            let a_column = (fi_vec - &f0_vec) / delta;
            a_matrix.set_column(i, &a_column);
        }

        println!("Calculating B Matrix (Jacobian wrt Control Inputs)...");
        for j in 0..n_inputs {
            let mut u_perturbed = u_trim.clone();
            // *** Restore perturbation logic ***
            match j {
                0 => u_perturbed.elevator += delta,
                1 => u_perturbed.aileron += delta,
                2 => u_perturbed.rudder += delta,
                3 => u_perturbed.throttle += delta,
                _ => unreachable!(),
            }
            let fj = calculate_flyer_state_derivatives(
                &x_trim,
                &u_perturbed,
                ac_config,
                &phys_config,
                &env_model,
            );
            let fj_vec = fj.to_vector();
            let b_column = (fj_vec - &f0_vec) / delta;
            b_matrix.set_column(j, &b_column);
        }

        println!("\n--- Linearization Complete ---");
        linearization_result.a_matrix = Some(a_matrix.clone());
        linearization_result.b_matrix = Some(b_matrix.clone());
        linearization_result.x_trim = Some(x_trim);
        linearization_result.u_trim = Some(u_trim);
        linearization_result.completed = true;

        if let Err(e) = save_matrices_to_csv(&a_matrix, &b_matrix) {
            error!("Failed to save matrices to CSV: {}", e);
        }

        println!("\nLinearization finished. Exiting application.");
        // *** Correct AppExit usage ***
        exit.send(AppExit::Success);
    } else if query.iter().count() > 1 {
        error!("Multiple TrackedAircraft found. Linearization requires a single target.");
        // *** Correct AppExit usage ***
        exit.send(AppExit::Success); // Or AppExit::Error if appropriate
    } // else: No tracked aircraft found yet or trim not started/complete - do nothing this frame
}

// --- CSV Saving Functions - Restore Implementations ---
fn write_matrix_csv<
    W: Write,
    R: nalgebra::Dim,
    C: nalgebra::Dim,
    S: nalgebra::Storage<f64, R, C>,
>(
    writer: &mut BufWriter<W>,
    matrix: &nalgebra::Matrix<f64, R, C, S>,
    title: &str,
    row_labels: &[&str],
    col_labels: &[&str],
) -> std::io::Result<()> {
    // Add return type back
    writeln!(writer, "{}", title)?;
    write!(writer, "State/Input")?;
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
    Ok(()) // Add Ok(()) back
}

fn save_matrices_to_csv(a_matrix: &DMatrix<f64>, b_matrix: &DMatrix<f64>) -> std::io::Result<()> {
    // Add return type back
    let state_labels = [
        "u (m/s)",
        "v (m/s)",
        "w (m/s)",
        "p (rad/s)",
        "q (rad/s)",
        "r (rad/s)",
        "phi (rad)",
        "theta (rad)",
        "psi (rad)",
        "pos_x (m)",
        "pos_y (m)",
        "pos_z (m)",
    ];
    let input_labels = ["elevator", "aileron", "rudder", "throttle"];

    let file = File::create("linearized_matrices.csv")?;
    let mut writer = BufWriter::new(file);

    write_matrix_csv(
        &mut writer,
        a_matrix,
        "A Matrix (System Dynamics - Jacobian wrt State)",
        &state_labels,
        &state_labels,
    )?; // Propagate error

    write_matrix_csv(
        &mut writer,
        b_matrix,
        "B Matrix (Control Effectiveness - Jacobian wrt Inputs)",
        &state_labels,
        &input_labels,
    )?; // Propagate error

    writer.flush()?;
    println!("Matrices saved successfully to linearized_matrices.csv");
    Ok(()) // Add Ok(()) back
}
