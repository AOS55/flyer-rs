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
    plugins::{LatestFrame, RenderCompleteEvent, RenderRequestEvent},
    server::ServerState,
};

pub fn render_frame(
    mut render_events: EventReader<RenderRequestEvent>,
    latest_frame: Res<LatestFrame>,
    mut complete_events: EventWriter<RenderCompleteEvent>,
) {
    for _event in render_events.read() {
        complete_events.send(RenderCompleteEvent {
            frame: latest_frame.data.clone(),
        });
    }
}

pub fn handle_render_response(
    mut server: ResMut<ServerState>,
    mut complete_events: EventReader<RenderCompleteEvent>,
) {
    for event in complete_events.read() {
        info!(
            "Processing render event with frame size: {}",
            event.frame.len()
        );

        if event.frame.is_empty() {
            error!("Empty frame data received");
            continue;
        }

        // Limit base64 encoding to reasonable size
        if event.frame.len() > 10_000_000 {
            // 10MB limit
            error!("Frame too large: {} bytes", event.frame.len());
            continue;
        }

        let response = match (|| {
            let guard = server
                .conn
                .lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            let mut stream = guard
                .try_clone()
                .map_err(|e| format!("Clone error: {}", e))?;

            let base64_frame = base64::encode(&event.frame);
            info!("Base64 frame size: {}", base64_frame.len());

            let response = serde_json::json!({
                "frame": base64_frame,
                "width": server.config.agent_config.render_width,
                "height": server.config.agent_config.render_height,
            });

            let response_str = serde_json::to_string(&response)? + "\n";
            stream.write_all(response_str.as_bytes())?;
            stream.flush()?;
            Ok::<_, Box<dyn std::error::Error>>(())
        })() {
            Ok(_) => info!("Render response sent successfully"),
            Err(e) => error!("Failed to send render response: {}", e),
        };
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
