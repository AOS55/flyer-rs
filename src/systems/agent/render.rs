use base64::Engine;
use bevy::prelude::*;
use std::io::Write;

use crate::{
    plugins::{FrameState, LatestFrame, RenderCompleteEvent, SimState},
    server::ServerState,
};

// Gets a Render of the environment, must await the update completing
pub fn render_frame(
    latest_frame: Res<LatestFrame>,
    mut frame_state: ResMut<FrameState>,
    mut complete_events: EventWriter<RenderCompleteEvent>,
) {
    if !frame_state.new_frame_available {
        warn!("Waiting for render frame...");
        return;
    }
    info!(
        "Render frame available! Frame size = {}",
        latest_frame.data.len()
    );
    complete_events.send(RenderCompleteEvent {
        frame: latest_frame.data.clone(),
    });
    frame_state.new_frame_available = false;
}

// Should only run once RenderCompleteEvent is received
pub fn handle_render_response(
    mut server: ResMut<ServerState>,
    mut complete_events: EventReader<RenderCompleteEvent>,
) {
    let conn = server.conn.clone();
    for event in complete_events.read() {
        info!(
            "Processing render event with frame size: {}",
            event.frame.len()
        );

        if event.frame.is_empty() {
            error!("Empty frame data received");
            continue;
        }

        if event.frame.len() > 10_000_000 {
            error!("Frame too large: {} bytes", event.frame.len());
            continue;
        }

        if let Ok(guard) = conn.lock() {
            if let Ok(mut stream) = guard.try_clone() {
                // Enable TCP_NODELAY
                if let Err(e) = stream.set_nodelay(true) {
                    error!("Failed to set TCP_NODELAY: {}", e);
                    continue;
                }

                let base64_frame = base64::prelude::BASE64_STANDARD.encode(&event.frame);
                let response = serde_json::json!({
                    "frame": base64_frame,
                    "width": server.config.agent_config.render_width,
                    "height": server.config.agent_config.render_height,
                });

                match serde_json::to_string(&response) {
                    Ok(response_str) => {
                        let len_bytes = (response_str.len() as u32).to_be_bytes();

                        // Write length prefix and data with error handling
                        if let Err(e) = stream.write_all(&len_bytes) {
                            error!("Failed to write length prefix: {}", e);
                            continue;
                        }

                        if let Err(e) = stream.write_all(response_str.as_bytes()) {
                            error!("Failed to write response data: {}", e);
                            continue;
                        }

                        if let Err(e) = stream.flush() {
                            error!("Failed to flush stream: {}", e);
                            continue;
                        }

                        info!("Successfully sent response of size: {}", response_str.len());
                        server.sim_state = SimState::WaitingForAction;
                    }
                    Err(e) => error!("Failed to serialize response: {}", e),
                }
            }
        }
    }
}
