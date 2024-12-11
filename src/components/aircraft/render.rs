use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct AircraftRenderState {
    pub attitude: Attitude,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum Attitude {
    UpRight,
    Right,
    DownRight,
    Up,
    Level,
    Down,
    UpLeft,
    LevelLeft,
    DownLeft,
}

impl Attitude {
    pub fn to_index(&self) -> usize {
        match self {
            Attitude::UpRight => 0,
            Attitude::Right => 1,
            Attitude::DownRight => 2,
            Attitude::Up => 3,
            Attitude::Level => 4,
            Attitude::Down => 5,
            Attitude::UpLeft => 6,
            Attitude::LevelLeft => 7,
            Attitude::DownLeft => 8,
        }
    }

    pub fn from_angles(pitch: f64, roll: f64) -> Attitude {
        const PITCH_THRESHOLD: f64 = std::f64::consts::PI / 18.0; // 10 degrees
        const ROLL_THRESHOLD: f64 = std::f64::consts::PI / 36.0; // 5 degrees

        match (pitch, roll) {
            // Nose up attitudes
            (p, r) if p > PITCH_THRESHOLD && r < -ROLL_THRESHOLD => Attitude::UpLeft,
            (p, r) if p > PITCH_THRESHOLD && r > ROLL_THRESHOLD => Attitude::UpRight,
            (p, _) if p > PITCH_THRESHOLD => Attitude::Up,

            // Nose down attitudes
            (p, r) if p < -PITCH_THRESHOLD && r < -ROLL_THRESHOLD => Attitude::DownLeft,
            (p, r) if p < -PITCH_THRESHOLD && r > ROLL_THRESHOLD => Attitude::DownRight,
            (p, _) if p < -PITCH_THRESHOLD => Attitude::Down,

            // Level attitudes
            (_, r) if r < -ROLL_THRESHOLD => Attitude::LevelLeft,
            (_, r) if r > ROLL_THRESHOLD => Attitude::Right,
            _ => Attitude::Level,
        }
    }
}
