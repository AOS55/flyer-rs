use bevy::prelude::*;
use dirs::data_local_dir;
use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use flyer::{
    plugins::{
        RenderCompleteEvent, RenderRequestEvent, ResetCompleteEvent, ResetRequestEvent, SimState,
        StepCompleteEvent, StepRequestEvent,
    },
    server::{setup_app, Command, EnvConfig, ServerState},
    systems::{
        aero_force_system, air_data_system, calculate_reward, collect_state, determine_terminated,
        dubins_aircraft_system, force_calculator_system, handle_render_response,
        handle_reset_response, handle_trim_requests, physics_integrator_system, propulsion_system,
        render_frame, reset_env, running_physics, sending_response, trim_aircraft_system,
        waiting_for_action,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Bevy server...");

    // Start TCP server
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    println!("PORT={}", listener.local_addr().unwrap().port());

    // Accept one connection
    let (stream, _addr) = listener.accept().unwrap();
    let stream = Arc::new(Mutex::new(stream));

    // Wait for initial config
    println!("Waiting for initial config...");
    let config = receive_initial_config(&stream)?;
    println!("Initial config received successfully");

    // Convert config to EnvConfig
    println!("Converting config to EnvConfig...");
    let env_config = EnvConfig::from_json(&config)?;
    println!("Config converted successfully");

    // Send ready signal
    {
        let aircraft_info: Vec<_> = env_config
            .aircraft_configs
            .keys()
            .map(|name| {
                serde_json::json!({
                    "name": name,
                    "config": env_config.aircraft_configs.get(name).unwrap(),
                    "action_space": env_config.action_spaces.get(name).unwrap(),
                    "observation_space": env_config.observation_spaces.get(name).unwrap()
                })
            })
            .collect();

        let response = serde_json::json!({
            "status": "ready",
            "aircraft": aircraft_info
        });
        let response_str = serde_json::to_string(&response)? + "\n";
        match stream.lock() {
            Ok(guard) => {
                if let Ok(mut clone) = guard.try_clone() {
                    clone.write_all(response_str.as_bytes())?;
                    clone.flush()?;
                    info!("Ready signal sent successfully: {}", response_str.trim());
                }
            }
            Err(e) => warn!("Failed to acquire stream lock: {}", e),
        }
    }

    // Create and configure bevy app
    info!("Initializing Bevy app...");
    let mut app = App::new();

    // Add server state resource
    app.insert_resource(ServerState {
        conn: stream.clone(),
        initialized: false,
        config: env_config.clone(),
        sim_state: SimState::WaitingForAction,
    });

    // Configure asset directory
    let asset_path = get_asset_path();
    info!("Using asset path: {}", asset_path.display());

    app = setup_app(
        app,
        env_config.clone(),
        asset_path.to_string_lossy().to_string(),
    );

    // Mark the server state as initialized
    app.world_mut()
        .get_resource_mut::<ServerState>()
        .unwrap()
        .initialized = true;

    // Add event and systems for handling step requests
    app
        // Command handling in PreUpdate
        // .add_systems(FixedFirst, debug_state)
        .add_systems(
            FixedPreUpdate,
            (handle_commands.run_if(waiting_state), handle_trim_requests),
        )
        // Action handling and Physics in Update
        .add_systems(
            FixedUpdate,
            (
                waiting_for_action.run_if(waiting_state),
                (
                    running_physics,
                    dubins_aircraft_system,
                    // aircraft_render_system,
                )
                    .chain()
                    .run_if(running_state),
                (
                    air_data_system,
                    aero_force_system,
                    propulsion_system,
                    force_calculator_system,
                    physics_integrator_system,
                )
                    .chain()
                    .run_if(running_state),
                // Add trim solver system here, after physics integration
                trim_aircraft_system.run_if(running_state),
                // --- End Add Trim System ---
                calculate_reward.run_if(running_state),
                determine_terminated.run_if(running_state),
                collect_state.run_if(sending_state),
                sending_response.run_if(sending_state),
            )
                .chain(),
        )
        // Render systems
        .add_systems(
            FixedUpdate,
            (render_frame, handle_render_response)
                .chain()
                .run_if(rendering),
        )
        // Events
        .add_event::<StepRequestEvent>()
        .add_event::<StepCompleteEvent>()
        .add_event::<ResetRequestEvent>()
        .add_event::<ResetCompleteEvent>()
        .add_event::<RenderRequestEvent>()
        .add_event::<RenderCompleteEvent>()
        .add_systems(FixedUpdate, reset_env.run_if(resetting_state));

    // Reset handling
    app.add_systems(
        FixedPostUpdate,
        handle_reset_response.run_if(resetting_state),
    );

    // Run app
    info!("Starting Bevy app...");
    app.run();

    Ok(())
}

// fn debug_state(current_state: ResMut<ServerState>) {
//     info!("CURRENT STATE: {:?}", current_state.sim_state);
// }

fn waiting_state(state: Res<ServerState>) -> bool {
    state.sim_state == SimState::WaitingForAction
}

fn running_state(state: Res<ServerState>) -> bool {
    state.sim_state == SimState::RunningPhysics
}

fn sending_state(state: Res<ServerState>) -> bool {
    state.sim_state == SimState::SendingResponse
}

fn resetting_state(state: Res<ServerState>) -> bool {
    state.sim_state == SimState::Resetting
}

fn rendering(state: Res<ServerState>) -> bool {
    state.sim_state == SimState::Rendering
}

fn get_asset_path() -> PathBuf {
    // Priority 1: User-defined environment variable
    if let Ok(asset_path) = env::var("FLYER_ASSETS_PATH") {
        return PathBuf::from(asset_path);
    }

    // Priority 2: User data directory (cross-platform)
    if let Some(local_path) = data_local_dir() {
        let asset_dir = local_path.join("flyer/assets");
        if asset_dir.exists() {
            return asset_dir;
        }
    }

    // Priority 3: System-wide location
    let system_asset_path = Path::new("/usr/local/share/flyer/assets");
    if system_asset_path.exists() {
        return system_asset_path.to_path_buf();
    }

    // Priority 4: Relative path (for development)
    let current_dir = env::current_dir().unwrap();
    let dev_asset_path = current_dir.join("flyer-rs/assets");
    if dev_asset_path.exists() {
        return dev_asset_path;
    }

    // If all else fails, return a reasonable fallback
    warn!("Assets not found in any standard location");
    PathBuf::from("assets")
}

/// Function to receive the initial configuration from the client.
///
/// # Arguments
/// * `stream` - The stream to receive data from.
///
/// # Returns
/// * `Result<serde_json::Value, std::io::Error>` - The configuration data or an error.
fn receive_initial_config(stream: &Arc<Mutex<TcpStream>>) -> std::io::Result<serde_json::Value> {
    info!("Attempting to receive initial config...");
    let guard = match stream.lock() {
        Ok(guard) => guard,
        Err(e) => {
            warn!("Failed to acquire stream lock: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire lock",
            ));
        }
    };

    let mut reader = BufReader::new(guard.try_clone()?);
    let mut line = String::new();

    info!("Reading line from stream...");
    match reader.read_line(&mut line) {
        Ok(n) => info!("Read {} bytes", n),
        Err(e) => warn!("Error reading line: {}", e),
    }
    info!("Received raw line: '{}'", line);

    let cmd: Command = match serde_json::from_str(&line) {
        Ok(cmd) => cmd,
        Err(e) => {
            warn!("Failed to parse command: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }
    };

    match cmd {
        Command::Initialize { config } => {
            info!("Got Initialize command with config");
            Ok(config)
        }
        _ => {
            let err = format!("Unexpected command received: {:?}", cmd);
            warn!("{}", err);
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        }
    }
}

fn handle_commands(
    mut server: ResMut<ServerState>,
    mut step_events: EventWriter<StepRequestEvent>,
    mut reset_events: EventWriter<ResetRequestEvent>,
) {
    let cmd = {
        let guard = server.conn.lock().unwrap();
        let stream = guard.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        if reader.read_line(&mut line).is_ok() && !line.is_empty() {
            match serde_json::from_str::<Command>(&line) {
                Ok(cmd) => Some(cmd),
                Err(e) => {
                    error!("Failed to parse command: {}", e);
                    if let Ok(mut stream) = guard.try_clone() {
                        let error_response = serde_json::json!({
                            "error": format!("Invalid command format: {}", e)
                        });
                        let response_str = serde_json::to_string(&error_response).unwrap() + "\n";
                        stream.write_all(response_str.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                    None
                }
            }
        } else {
            None
        }
    };

    if let Some(cmd) = cmd {
        // Only process commands if we're not in RunningPhysics state
        if server.sim_state != SimState::RunningPhysics {
            match cmd {
                Command::Initialize { .. } => {
                    // Handle late initialization attempts
                    if server.initialized {
                        warn!("Server already initialized, ignoring Initialize command");
                        if let Ok(guard) = server.conn.lock() {
                            if let Ok(mut stream) = guard.try_clone() {
                                let response = serde_json::json!({
                                    "error": "Server already initialized"
                                });
                                let response_str = serde_json::to_string(&response).unwrap() + "\n";
                                stream.write_all(response_str.as_bytes()).unwrap();
                                stream.flush().unwrap();
                            }
                        }
                    }
                }

                Command::Step { actions } => {
                    if server.sim_state == SimState::WaitingForAction {
                        info!("Step Command Received!");
                        info!("actions: {:?}", actions);
                        step_events.send(StepRequestEvent { actions });
                        // State transition will be handled by waiting_for_action system
                    } else {
                        warn!(
                            "Received Step command while in {:?} state",
                            server.sim_state
                        );
                    }
                }

                Command::Reset { seed } => {
                    info!("Reset Command Received with seed: {:?}", seed);

                    // Rebuild EnvConfig with new seed if provided
                    if let Some(seed_value) = seed {
                        match server.config.rebuild_with_seed(seed_value) {
                            Ok(new_config) => {
                                server.config = new_config;
                                info!("Successfully rebuilt EnvConfig with seed: {}", seed_value);
                                info!("Server Config: {:?}", server.config);
                            }
                            Err(e) => {
                                error!("Failed to rebuild EnvConfig: {}", e);
                                if let Ok(guard) = server.conn.lock() {
                                    if let Ok(mut stream) = guard.try_clone() {
                                        let error_response = serde_json::json!({
                                            "error": format!("Failed to rebuild config with seed {}: {}", seed_value, e)
                                        });
                                        let response_str =
                                            serde_json::to_string(&error_response).unwrap() + "\n";
                                        stream.write_all(response_str.as_bytes()).unwrap();
                                        stream.flush().unwrap();
                                    }
                                }
                                return;
                            }
                        }
                    }

                    // Send reset event and transition to Resetting state
                    info!("Transitioning to Resetting state");
                    reset_events.send(ResetRequestEvent { seed });
                    server.sim_state = SimState::Resetting;
                }

                Command::Render => {
                    info!("Render command received");
                    // render_events.send(RenderRequestEvent);
                    server.sim_state = SimState::Rendering;
                }

                Command::Close => {
                    info!("Close command received");
                    if let Ok(guard) = server.conn.lock() {
                        if let Ok(mut stream) = guard.try_clone() {
                            let response = "Close command acknowledged";
                            let response_str = serde_json::to_string(&response).unwrap() + "\n";
                            stream.write_all(response_str.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                    }
                    // TODO: Implement clean shutdown
                    // Could send a shutdown event or set a shutdown flag
                }
            }
        } else {
            warn!("Command received while in RunningPhysics state, ignoring");
        }
    }
}
