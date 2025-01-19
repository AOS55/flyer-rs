use bevy::prelude::*;
use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use flyer::{
    plugins::{
        ResetCompleteEvent, ResetRequestEvent, SimState, StepCompleteEvent, StepRequestEvent,
    },
    server::{setup_app, Command, EnvConfig, ServerState},
    systems::{
        aero_force_system, air_data_system, calculate_reward, determine_terminated,
        dubins_aircraft_system, force_calculator_system, handle_reset_response,
        physics_integrator_system, reset_env, running_physics, sending_response,
        waiting_for_action,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    // setup_logging();

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
                    println!("Ready signal sent successfully: {}", response_str.trim());
                }
            }
            Err(e) => eprintln!("Failed to acquire stream lock: {}", e),
        }
    }

    // Create and configure bevy app
    println!("Initializing Bevy app...");
    let mut app = App::new();

    // Add server state resource
    app.insert_resource(ServerState {
        conn: stream.clone(),
        initialized: false,
        config: env_config.clone(),
        sim_state: SimState::WaitingForAction,
    });

    // Configure asset directory
    let current_dir = env::current_dir().unwrap();
    let asset_path = current_dir
        .join("flyer-rs/assets")
        .to_str()
        .unwrap()
        .to_string();

    app = setup_app(app, env_config.clone(), asset_path);

    // Mark the server state as initialized
    app.world_mut()
        .get_resource_mut::<ServerState>()
        .unwrap()
        .initialized = true;

    // Add event and systems for handling step requests
    app
        // Command handling in PreUpdate
        .add_systems(FixedFirst, debug_state)
        .add_systems(FixedPreUpdate, handle_commands.run_if(waiting_state))
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
                    force_calculator_system,
                    physics_integrator_system,
                )
                    .chain()
                    .run_if(running_state),
                calculate_reward.run_if(running_state),
                determine_terminated.run_if(running_state),
                sending_response.run_if(sending_state),
            )
                .chain(),
        )
        // Events
        .add_event::<StepRequestEvent>()
        .add_event::<StepCompleteEvent>()
        .add_event::<ResetRequestEvent>()
        .add_event::<ResetCompleteEvent>()
        // Reset handling
        .add_systems(FixedUpdate, reset_env);

    // Add event for handling reset requests
    app.add_systems(
        FixedPostUpdate,
        handle_reset_response.run_if(resetting_state),
    );

    // Run app
    println!("Starting Bevy app...");
    app.run();

    Ok(())
}

fn debug_state(current_state: ResMut<ServerState>) {
    info!("CURRENT STATE: {:?}", current_state.sim_state);
}

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
            eprintln!("Failed to acquire stream lock: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to acquire lock",
            ));
        }
    };

    let mut reader = BufReader::new(guard.try_clone()?);
    let mut line = String::new();

    println!("Reading line from stream...");
    match reader.read_line(&mut line) {
        Ok(n) => println!("Read {} bytes", n),
        Err(e) => eprintln!("Error reading line: {}", e),
    }
    println!("Received raw line: '{}'", line);

    let cmd: Command = match serde_json::from_str(&line) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Failed to parse command: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }
    };

    match cmd {
        Command::Initialize { config } => {
            println!("Got Initialize command with config");
            Ok(config)
        }
        _ => {
            let err = format!("Unexpected command received: {:?}", cmd);
            eprintln!("{}", err);
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
