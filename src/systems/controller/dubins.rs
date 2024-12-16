use bevy::prelude::*;

use crate::components::{DubinsAircraftControls, DubinsAircraftState, PlayerController};

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

pub fn dubins_gym_control_system(
    mut query: Query<&mut DubinsAircraftState, With<PlayerController>>,
    action: DubinsAircraftControls,
) {
    if let Ok(mut aircraft_state) = query.get_single_mut() {
        aircraft_state.controls = action;
    }
}
