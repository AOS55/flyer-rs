// use crate::{plugins::RenderRequestEvent, resources::AgentState};
// use bevy::{
//     prelude::*,
//     render::view::screenshot::{Screenshot, ScreenshotCaptured},
//     window::PrimaryWindow,
// };

use base64;
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

    // for _event in render_events.read() {
    //     warn!("Render frame unavailable!");
    //     if !frame_state.new_frame_available {
    //         info!("Exiting and trying again next frame");
    //         return; // Exit and try again next frame
    //     }
    //     info!(
    //         "Render frame: latest_frame size = {}",
    //         latest_frame.data.len()
    //     );
    //     complete_events.send(RenderCompleteEvent {
    //         frame: latest_frame.data.clone(),
    //     });

    //     // Reset the flag after sending the frame
    //     frame_state.new_frame_available = false;
    // }
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

                let base64_frame = base64::encode(&event.frame);
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

// Old method to capture a screenshot from a running system (for render() API method)
// // Tracks whether a screenshot has been triggered
// #[derive(Resource, Default)]
// pub struct ScreenshotState {
//     triggered: bool,
// }

// /// System for capturing and processing screenshots.
// ///
// /// This system triggers a screenshot of the primary window and captures its pixel data
// /// when the screenshot is ready. The captured image data is stored in the `AgentState`
// /// resource's render buffer for further processing or analysis.
// pub fn capture_frame(
//     mut commands: Commands,
//     mut events: EventReader<ScreenshotCaptured>,
//     windows: Query<Entity, With<PrimaryWindow>>,
//     mut screenshot_state: ResMut<ScreenshotState>,
//     agent_state: ResMut<AgentState>,
// ) {
//     // Step 1: Trigger a screenshot once
//     if !screenshot_state.triggered {
//         if let Ok(window_entity) = windows.get_single() {
//             commands.spawn(Screenshot::window(window_entity));
//             screenshot_state.triggered = true;
//             println!("Screenshot triggered!");
//         }
//     }

//     // Step 2: Capture the screenshot when ready
//     for event in events.read() {
//         let image = &event.0; // Access the `Image` from ScreenshotCaptured

//         println!(
//             "Screenshot captured: width={}, height={}, bytes={}",
//             image.texture_descriptor.size.width,
//             image.texture_descriptor.size.height,
//             image.data.len(),
//         );

//         // Add the image byte data to the render buffer
//         let mut render_buffer = agent_state.render_buffer.lock().unwrap();
//         *render_buffer = Some(image.data.clone());

//         // Reset to allow triggering another screenshot
//         screenshot_state.triggered = false;

//         println!("Screenshot processing complete!");
//     }
// }
