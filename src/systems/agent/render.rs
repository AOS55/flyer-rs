use crate::resources::AgentState;
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::PrimaryWindow,
};

// Tracks whether a screenshot has been triggered
#[derive(Resource, Default)]
pub struct ScreenshotState {
    triggered: bool,
}

/// System for capturing and processing screenshots.
///
/// This system triggers a screenshot of the primary window and captures its pixel data
/// when the screenshot is ready. The captured image data is stored in the `AgentState`
/// resource's render buffer for further processing or analysis.
pub fn capture_frame(
    mut commands: Commands,
    mut events: EventReader<ScreenshotCaptured>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_state: ResMut<ScreenshotState>,
    agent_state: ResMut<AgentState>,
) {
    // Step 1: Trigger a screenshot once
    if !screenshot_state.triggered {
        if let Ok(window_entity) = windows.get_single() {
            commands.spawn(Screenshot::window(window_entity));
            screenshot_state.triggered = true;
            println!("Screenshot triggered!");
        }
    }

    // Step 2: Capture the screenshot when ready
    for event in events.read() {
        let image = &event.0; // Access the `Image` from ScreenshotCaptured

        println!(
            "Screenshot captured: width={}, height={}, bytes={}",
            image.texture_descriptor.size.width,
            image.texture_descriptor.size.height,
            image.data.len(),
        );

        // Add the image byte data to the render buffer
        let mut render_buffer = agent_state.render_buffer.lock().unwrap();
        *render_buffer = Some(image.data.clone());

        // Reset to allow triggering another screenshot
        screenshot_state.triggered = false;

        println!("Screenshot processing complete!");
    }
}
