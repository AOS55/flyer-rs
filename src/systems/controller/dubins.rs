use bevy::prelude::*;

use crate::components::{DubinsAircraftControls, DubinsAircraftState, PlayerController};

/// System for controlling a Dubins aircraft using keyboard input.
///
/// This system maps specific keyboard inputs to control the aircraft's acceleration,
/// bank angle, and vertical speed. The controls are only applied to the player-controlled aircraft.
pub fn dubins_keyboard_system(
    mut query: Query<&mut DubinsAircraftState, With<PlayerController>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut aircraft_state) = query.get_single_mut() {
        if keyboard.pressed(KeyCode::ArrowUp) {
            aircraft_state.controls.acceleration += 0.1
        } else if keyboard.pressed(KeyCode::ArrowDown) {
            aircraft_state.controls.acceleration -= 0.1
        } else if keyboard.pressed(KeyCode::ArrowLeft) {
            aircraft_state.controls.bank_angle -= 0.01
        } else if keyboard.pressed(KeyCode::ArrowRight) {
            aircraft_state.controls.bank_angle += 0.01
        } else if keyboard.pressed(KeyCode::KeyW) {
            aircraft_state.controls.vertical_speed += 0.1
        } else if keyboard.pressed(KeyCode::KeyS) {
            aircraft_state.controls.vertical_speed -= 0.1
        }
    }
}

/// System for controlling a Dubins aircraft via external actions (e.g., gym interface).
///
/// This system applies externally provided control inputs (e.g., from reinforcement learning
/// agents or simulation scripts) to the player-controlled aircraft. It replaces the current
/// controls with the provided action.
pub fn dubins_gym_control_system(
    mut query: Query<&mut DubinsAircraftState, With<PlayerController>>,
    action: DubinsAircraftControls,
) {
    if let Ok(mut aircraft_state) = query.get_single_mut() {
        aircraft_state.controls = action;
    }
}
